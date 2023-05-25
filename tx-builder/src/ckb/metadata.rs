use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use axon_types::metadata::MetadataCellData as AMetadataCellData;
use ckb_types::{
    core::{Capacity, TransactionBuilder, TransactionView},
    packed::{CellOutput, Script},
    prelude::{Entity, Pack},
};

use common::traits::{
    smt::{DelegateSmtStorage, ProposalSmtStorage, StakeSmtStorage},
    tx_builder::IMetadataTxBuilder,
};
use common::types::tx_builder::*;
use ethereum_types::H160;

pub struct MetadataSmtTxBuilder<PSmt> {
    _kicker:         PrivateKey,
    _quorum:         u16,
    last_checkpoint: Checkpoint,
    last_metadata:   Metadata,
    smt:             PSmt,
}

#[async_trait]
impl<PSmt> IMetadataTxBuilder<PSmt> for MetadataSmtTxBuilder<PSmt>
where
    PSmt: ProposalSmtStorage + StakeSmtStorage + DelegateSmtStorage + Send + 'static + Sync,
{
    fn new(
        _kicker: PrivateKey,
        _quorum: u16,
        last_metadata: Metadata,
        last_checkpoint: Checkpoint,
        smt: PSmt,
    ) -> Self {
        Self {
            _kicker,
            _quorum,
            last_checkpoint,
            last_metadata,
            smt,
        }
    }

    async fn build_tx(&self) -> Result<(TransactionView, NonTopStakers, NonTopDelegators)> {
        // todo: get metadata cell, stake smt cell, delegate smt cell
        let inputs = vec![];

        ProposalSmtStorage::insert(
            &self.smt,
            self.last_checkpoint.epoch,
            self.last_checkpoint
                .propose_count
                .iter()
                .map(|v| (v.proposer.0.into(), v.count as u64))
                .collect(),
        )
        .await?;

        struct EpochStakeInfo {
            staker:     common::types::H160,
            amount:     u128,
            delegaters: HashMap<common::types::H160, u128>,
        }

        impl EpochStakeInfo {
            fn total_stake(&self) -> u128 {
                self.amount + self.delegaters.values().sum::<u128>()
            }
        }

        impl PartialOrd for EpochStakeInfo {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.total_stake().cmp(&other.total_stake()))
            }
        }

        impl Ord for EpochStakeInfo {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.total_stake().cmp(&other.total_stake())
            }
        }

        impl PartialEq for EpochStakeInfo {
            fn eq(&self, other: &Self) -> bool {
                self.staker == other.staker
                    && self.amount == other.amount
                    && self.delegaters == other.delegaters
            }
        }

        impl Eq for EpochStakeInfo {}
        let (validators, no_top_stakers, no_top_delegators) = {
            let stakers =
                StakeSmtStorage::get_sub_leaves(&self.smt, self.last_checkpoint.epoch).await?;

            let mut mid = std::collections::BinaryHeap::new();

            for (staker, amount) in stakers {
                let delegaters = DelegateSmtStorage::get_sub_leaves(
                    &self.smt,
                    self.last_checkpoint.epoch,
                    staker.clone(),
                )
                .await?;
                mid.push(EpochStakeInfo {
                    staker,
                    amount,
                    delegaters,
                })
            }

            let stake_infos = {
                let mut res = Vec::with_capacity(self._quorum as usize);
                for _ in 0..self._quorum {
                    res.push(mid.pop().unwrap())
                }
                res
            };

            if mid.is_empty() {
                (stake_infos, Vec::default(), HashMap::default())
            } else {
                let mut no_top_stakers = Vec::with_capacity(mid.len());
                let mut no_top_delegaters: HashMap<H160, HashMap<H160, u128>> = HashMap::default();

                for i in mid {
                    no_top_stakers.push((i.staker, i.amount));
                    for (d, u) in i.delegaters {
                        no_top_delegaters
                            .entry(d.0.into())
                            .or_default()
                            .insert(i.staker.0.into(), u);
                    }
                }

                (stake_infos, no_top_stakers, no_top_delegaters)
            }
        };

        StakeSmtStorage::remove(
            &self.smt,
            self.last_checkpoint.epoch,
            no_top_stakers.iter().map(|(a, _)| a).cloned().collect(),
        )
        .await
        .unwrap();

        StakeSmtStorage::new_epoch(&self.smt, self.last_checkpoint.epoch + 1)
            .await
            .unwrap();

        let old_stake_smt_proof = StakeSmtStorage::generate_sub_proof(
            &self.smt,
            self.last_checkpoint.epoch - 1,
            self.last_metadata
                .validators
                .iter()
                .map(|v| v.address.0.into())
                .collect(),
        )
        .await
        .unwrap();
        let new_stake_smt_proof = StakeSmtStorage::generate_sub_proof(
            &self.smt,
            self.last_checkpoint.epoch,
            validators.iter().map(|v| v.staker).collect(),
        )
        .await
        .unwrap();

        let stake_smt_update = {
            StakeSmtUpdateInfo {
                all_stake_infos: {
                    let mut res = Vec::with_capacity(validators.len());

                    for v in validators.iter() {
                        res.push(StakeInfo {
                            addr:   v.staker.0.into(),
                            amount: v.amount,
                        })
                    }

                    res
                },
                old_epoch_proof: old_stake_smt_proof,
                new_epoch_proof: new_stake_smt_proof,
            }
        };

        let new_stake_root = StakeSmtStorage::get_top_root(&self.smt).await.unwrap();

        let delegatets_remove_keys = no_top_delegators
            .iter()
            .map(|(d, i)| i.keys().cloned().zip(std::iter::repeat(d.clone())))
            .flatten()
            .collect();

        DelegateSmtStorage::remove(
            &self.smt,
            self.last_checkpoint.epoch,
            delegatets_remove_keys,
        )
        .await
        .unwrap();

        DelegateSmtStorage::new_epoch(&self.smt, self.last_checkpoint.epoch + 1)
            .await
            .unwrap();

        let mut delegator_smt_update_infos = Vec::with_capacity(validators.len());
        let mut delegator_staker_smt_roots = Vec::with_capacity(validators.len());
        for v in validators.iter() {
            delegator_staker_smt_roots.push(StakerSmtRoot {
                staker: v.staker.0.into(),
                root:   Into::<[u8; 32]>::into(
                    DelegateSmtStorage::get_sub_root(
                        &self.smt,
                        self.last_checkpoint.epoch,
                        v.staker,
                    )
                    .await
                    .unwrap()
                    .unwrap(),
                )
                .into(),
            });
            delegator_smt_update_infos.push(StakeGroupInfo {
                staker:                   v.staker.0.into(),
                delegate_infos:           v
                    .delegaters
                    .iter()
                    .map(|v| DelegateInfo {
                        delegator_addr: v.0 .0.into(),
                        amount:         *v.1,
                    })
                    .collect(),
                delegate_old_epoch_proof: DelegateSmtStorage::generate_sub_proof(
                    &self.smt,
                    v.staker,
                    self.last_checkpoint.epoch - 1,
                    v.delegaters.iter().map(|v| v.0 .0.into()).collect(),
                )
                .await
                .unwrap(),
                delegate_new_epoch_proof: DelegateSmtStorage::generate_sub_proof(
                    &self.smt,
                    v.staker,
                    self.last_checkpoint.epoch,
                    v.delegaters.iter().map(|v| v.0 .0.into()).collect(),
                )
                .await
                .unwrap(),
            })
        }

        let new_metadata = {
            let mut new_validators: Vec<Validator> = Vec::with_capacity(validators.len());
            for v in validators {
                new_validators.push(Validator {
                    // todo: fetch validators' blspubkey pubkey to build metadata
                    bls_pub_key:    bytes::Bytes::default(),
                    // todo: fetch validators' blspubkey pubkey to build metadata
                    pub_key:        bytes::Bytes::default(),
                    address:        v.staker.0.into(),
                    // todo
                    propose_weight: 0,
                    // todo
                    vote_weight:    0,
                    // new epoch start with all zero?
                    propose_count:  0,
                })
            }

            Metadata {
                epoch_len:       self.last_metadata.epoch_len,
                period_len:      self.last_metadata.period_len,
                quorum:          self._quorum,
                gas_limit:       self.last_metadata.gas_limit,
                gas_price:       self.last_metadata.gas_price,
                interval:        self.last_metadata.interval,
                validators:      new_validators,
                propose_ratio:   self.last_metadata.propose_ratio,
                prevote_ratio:   self.last_metadata.prevote_ratio,
                precommit_ratio: self.last_metadata.precommit_ratio,
                brake_ratio:     self.last_metadata.brake_ratio,
                tx_num_limit:    self.last_metadata.tx_num_limit,
                max_tx_size:     self.last_metadata.max_tx_size,
                block_height:    self.last_checkpoint.latest_block_height,
            }
        };

        // fetch metadata cell data
        let mut metadata_cell_data = MetadataCellData {
            metadata: vec![Metadata::default(), Metadata::default()],
            ..Default::default()
        };

        // update metadata
        metadata_cell_data.epoch = self.last_checkpoint.epoch + 2;
        metadata_cell_data.metadata.remove(0);
        metadata_cell_data.metadata.push(new_metadata);

        let stake_smt_cell_data = StakeSmtCellData {
            smt_root:         Into::<[u8; 32]>::into(new_stake_root).into(),
            version:          0,
            metadata_type_id: metadata_cell_data.type_ids.metadata_type_id.clone(),
        };

        let delegate_smt_cell_data = DelegateSmtCellData {
            version:          0,
            smt_roots:        delegator_staker_smt_roots,
            metadata_type_id: metadata_cell_data.type_ids.metadata_type_id.clone(),
        };

        // todo
        let outputs_data = vec![Into::<AMetadataCellData>::into(metadata_cell_data).as_bytes()];

        // todo: fill lock, type
        let fake_lock = Script::default();
        let fake_type = Script::default();
        let outputs = vec![
            // metadata cell
            CellOutput::new_builder()
                .lock(fake_lock.clone())
                .type_(Some(fake_type.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[0].len())?)?,
            // stake smt cell
            CellOutput::new_builder()
                .lock(fake_lock.clone())
                .type_(Some(fake_type.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[0].len())?)?,
            // delegate smt cell
            CellOutput::new_builder()
                .lock(fake_lock)
                .type_(Some(fake_type).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[0].len())?)?,
        ];

        // todo: add removed stakers' stake AT cells to inputs and outputs and
        //       add withdraw AT cells to outputs

        // todo: add removed delegators' delegate AT cells to inputs and outputs and
        //       add withdraw AT cells to outputs

        // todo
        let cell_deps = vec![];

        // todo: balance tx, fill placeholder witnesses,
        let tx = TransactionBuilder::default()
            .inputs(inputs)
            .outputs(outputs)
            .outputs_data(outputs_data.pack())
            .cell_deps(cell_deps)
            .build();

        // todo: sign tx

        Ok((tx, HashMap::default(), HashMap::default()))
    }
}
