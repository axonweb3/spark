use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use axon_types::{
    basic::Byte65, delegate::DelegateAtCellData as ADelegateAtCellData,
    metadata::MetadataCellData as AMetadataCellData,
    withdraw::WithdrawAtCellData as AWithdrawAtCellData,
};
use ckb_types::{
    core::{Capacity, TransactionBuilder, TransactionView},
    packed::{CellInput, CellOutput, Script},
    prelude::{Entity, Pack},
};

use common::traits::{
    smt::{DelegateSmtStorage, ProposalSmtStorage, StakeSmtStorage},
    tx_builder::IMetadataTxBuilder,
};
use common::types::tx_builder::*;
use ethereum_types::H160;

use crate::ckb::define::types::*;

pub struct MetadataSmtTxBuilder<PSmt> {
    _kicker:         PrivateKey,
    quorum:          u16,
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
        quorum: u16,
        last_metadata: Metadata,
        last_checkpoint: Checkpoint,
        smt: PSmt,
    ) -> Self {
        Self {
            _kicker,
            quorum,
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
                    staker,
                )
                .await?;
                mid.push(EpochStakeInfo {
                    staker,
                    amount,
                    delegaters,
                })
            }

            let stake_infos = {
                let mut res = Vec::with_capacity(self.quorum as usize);
                for _ in 0..3 * self.quorum {
                    match mid.pop() {
                        Some(s) => res.push(s),
                        None => break,
                    }
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

        let _stake_smt_update = {
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
            .flat_map(|(d, i)| i.keys().cloned().zip(std::iter::repeat(*d)))
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
                    pub_key:        Byte65::default(),
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
                quorum:          self.quorum,
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

        let _stake_smt_cell_data = StakeSmtCellData {
            smt_root:         Into::<[u8; 32]>::into(new_stake_root).into(),
            metadata_type_id: metadata_cell_data.type_ids.metadata_type_id.clone(),
        };

        let _delegate_smt_cell_data = DelegateSmtCellData {
            smt_roots:        delegator_staker_smt_roots,
            metadata_type_id: metadata_cell_data.type_ids.metadata_type_id.clone(),
        };

        let mut withdraw_set: HashMap<H160, u128> = HashMap::default();
        // remove no top staker
        let mut no_top_staker_cell_datas = Vec::with_capacity(no_top_stakers.len());
        let mut no_top_staker_cell_inputs: Vec<CellInput> =
            Vec::with_capacity(no_top_stakers.len());
        let mut no_top_staker_cell_outputs: Vec<CellOutput> =
            Vec::with_capacity(no_top_stakers.len());

        for (addr, amount) in no_top_stakers.iter() {
            *withdraw_set.entry(*addr).or_default() += amount;
            // todo: fetch staker AT cell data
            // AT cell data is u128 le bytes(total amount) + StakeAtCellData molecule data,
            // here just a mock no top staker just change these staker's AT cell
            // total amount
            let data = bytes::Bytes::from(Vec::from(999u128.to_le_bytes()));
            no_top_staker_cell_inputs.push(CellInput::default());
            no_top_staker_cell_outputs.push(CellOutput::default());

            let total_amount = {
                let mut total = [0u8; 16];
                total.copy_from_slice(&data[0..16]);
                u128::from_le_bytes(total) - amount
            };

            let new_data = {
                let mut res = total_amount.to_le_bytes().to_vec();
                res.extend_from_slice(&data[16..]);
                bytes::Bytes::from(res)
            };
            no_top_staker_cell_datas.push(new_data);
        }

        // remove no top delegator
        let mut no_top_delegator_cell_inputs: Vec<CellInput> =
            Vec::with_capacity(no_top_stakers.len());
        let mut delegator_at_cell_datas = HashMap::with_capacity(no_top_stakers.len());
        for (staker_address, v) in no_top_delegators.iter() {
            for (addr, amount) in v {
                *withdraw_set.entry(*addr).or_default() += amount;

                let (_cell_output, total_amount, delegator_at_cell_data) =
                    match delegator_at_cell_datas.entry(*addr) {
                        std::collections::hash_map::Entry::Occupied(v) => v.into_mut(),
                        std::collections::hash_map::Entry::Vacant(v) => {
                            // todo: fetch delegator AT cell data
                            // AT cell data is u128 le bytes(total amount) +
                            // DelegateAtCellData molecule
                            // data, here just a mock no top delegator just change
                            // these delegator's AT cell
                            // total amount
                            no_top_delegator_cell_inputs.push(CellInput::default());
                            v.insert((
                                CellOutput::default(),
                                999u128,
                                DelegateAtCellData::default(),
                            ))
                        }
                    };

                *total_amount -= amount;
                for i in delegator_at_cell_data.lock.delegator_infos.iter_mut() {
                    if i.staker == staker_address.0.into() {
                        i.total_amount -= amount;
                    }
                }
            }
        }

        let (_no_top_delegator_cell_outputs, _no_top_delegator_cell_output_datas): (
            Vec<CellOutput>,
            Vec<bytes::Bytes>,
        ) = delegator_at_cell_datas
            .into_values()
            .map(|(cell_output, total_amount, data)| {
                let mut res = total_amount.to_le_bytes().to_vec();
                res.extend((Into::<ADelegateAtCellData>::into(data)).as_slice());
                (cell_output, bytes::Bytes::from(res))
            })
            .unzip();

        let mut withdraw_inputs: Vec<CellInput> = Vec::with_capacity(withdraw_set.len());
        let mut withdraw_outputs: Vec<CellOutput> = Vec::with_capacity(withdraw_set.len());
        let mut withdraw_output_datas: Vec<bytes::Bytes> = Vec::with_capacity(withdraw_set.len());
        for (_addr, amount) in withdraw_set {
            // fetch withdraw cell data
            withdraw_inputs.push(CellInput::default());
            withdraw_outputs.push(CellOutput::default());
            let mut total_amount = 999u128;
            total_amount += amount;
            let mut withdraw_data = WithdrawAtCellData::default();
            if withdraw_data
                .lock
                .withdraw_infos
                .last()
                .map(|v| v.epoch == self.last_checkpoint.epoch)
                .unwrap_or_default()
            {
                withdraw_data.lock.withdraw_infos.last_mut().unwrap().amount += amount;
            } else {
                withdraw_data.lock.withdraw_infos.push(WithdrawInfo {
                    amount,
                    epoch: self.last_checkpoint.epoch,
                })
            }
            withdraw_output_datas.push({
                let mut res = total_amount.to_le_bytes().to_vec();
                res.extend(Into::<AWithdrawAtCellData>::into(withdraw_data).as_slice());
                bytes::Bytes::from(res)
            })
        }

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
