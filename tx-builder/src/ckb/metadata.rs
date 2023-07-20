use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use ckb_types::{
    core::{Capacity, TransactionBuilder, TransactionView},
    packed::{CellInput, CellOutput, WitnessArgs},
    prelude::{Entity, Pack, Reader},
};
use ethereum_types::H160;

use common::traits::{
    ckb_rpc_client::CkbRpc,
    smt::{DelegateSmtStorage, ProposalSmtStorage, StakeSmtStorage},
    tx_builder::IMetadataTxBuilder,
};
use common::types::axon_types::{
    delegate::{
        DelegateAtCellData as ADelegateAtCellData, DelegateSmtCellData as ADelegateSmtCellData,
    },
    metadata::{MetadataCellData as AMetadataCellData, MetadataWitness as AMetadataWitness},
    stake::{StakeAtCellData as AStakeAtCellData, StakeSmtCellData as AStakeSmtCellData},
    withdraw::WithdrawAtCellData as AWithdrawAtCellData,
};
use common::types::tx_builder::*;
use molecule::prelude::Builder;

use crate::ckb::define::constants::*;
use crate::ckb::define::types::*;
use crate::ckb::helper::{
    AlwaysSuccess, Checkpoint as HCheckpoint, Delegate as HDelegate, Metadata as HMetadata,
    OmniEth, Secp256k1, Stake as HStake, Withdraw, Xudt,
};

pub struct MetadataSmtTxBuilder<'a, C: CkbRpc, PSmt> {
    ckb:             &'a C,
    _kicker:         PrivateKey,
    quorum:          u16,
    type_ids:        TypeIds,
    last_checkpoint: Checkpoint,
    last_metadata:   Metadata,
    smt:             PSmt,
}

#[async_trait]
impl<'a, C: CkbRpc, PSmt> IMetadataTxBuilder<'a, C, PSmt> for MetadataSmtTxBuilder<'a, C, PSmt>
where
    PSmt: ProposalSmtStorage + StakeSmtStorage + DelegateSmtStorage + Send + 'static + Sync,
{
    fn new(
        ckb: &'a C,
        _kicker: PrivateKey,
        quorum: u16,
        type_ids: TypeIds,
        last_metadata: Metadata,
        last_checkpoint: Checkpoint,
        smt: PSmt,
    ) -> Self {
        Self {
            ckb,
            _kicker,
            quorum,
            type_ids,
            last_checkpoint,
            last_metadata,
            smt,
        }
    }

    async fn build_tx(
        &self,
    ) -> Result<(
        TransactionView,
        Vec<(H160, u128)>,
        HashMap<H160, HashMap<H160, u128>>,
    )> {
        let metadata_type = HMetadata::type_(&self.type_ids.metadata_type_id);

        let last_metadata_cell = HMetadata::get_cell(self.ckb, metadata_type.clone()).await?;

        let last_metadata_cell_data =
            AMetadataCellData::new_unchecked(last_metadata_cell.output_data.unwrap().into_bytes());

        let stake_smt = HStake::smt_type(&self.type_ids.stake_smt_type_id);

        let last_stake_smt_cell = HStake::get_smt_cell(self.ckb, stake_smt.clone()).await?;

        let delegate_smt = HDelegate::smt_type(&self.type_ids.delegate_smt_type_id);

        let last_delegate_smt_cell =
            HDelegate::get_smt_cell(self.ckb, delegate_smt.clone()).await?;

        let mut inputs = vec![
            // metadata
            CellInput::new_builder()
                .previous_output(last_metadata_cell.out_point.into())
                .build(),
            // stake smt
            CellInput::new_builder()
                .previous_output(last_stake_smt_cell.out_point.into())
                .build(),
            // delegate smt
            CellInput::new_builder()
                .previous_output(last_delegate_smt_cell.out_point.into())
                .build(),
        ];

        let mut witnesses = vec![
            WitnessArgs::default().as_bytes(),
            WitnessArgs::default().as_bytes(),
            WitnessArgs::default().as_bytes(),
        ];

        ProposalSmtStorage::insert(
            &self.smt,
            self.last_checkpoint.epoch,
            self.last_checkpoint
                .propose_count
                .iter()
                .map(|v| (v.proposer.0.into(), v.count))
                .collect(),
        )
        .await?;

        let proposal_count_smt_root = ProposalSmtStorage::get_top_root(&self.smt).await?;
        let new_proposal_count_smt_proof = ProposalSmtStorage::generate_sub_proof(
            &self.smt,
            self.last_checkpoint.epoch,
            self.last_checkpoint
                .propose_count
                .iter()
                .map(|v| v.proposer.0.into())
                .collect(),
        )
        .await
        .unwrap();

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

        let old_stake_smt_proof = StakeSmtStorage::generate_sub_proof(
            &self.smt,
            self.last_checkpoint.epoch,
            self.last_metadata
                .validators
                .iter()
                .map(|v| v.address.0.into())
                .collect(),
        )
        .await
        .unwrap();

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

        let new_stake_smt_proof = StakeSmtStorage::generate_sub_proof(
            &self.smt,
            self.last_checkpoint.epoch + 1,
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
                new_epoch_proof: new_stake_smt_proof.clone(),
            }
        };

        let new_stake_root = StakeSmtStorage::get_top_root(&self.smt).await.unwrap();

        let mut old_delegator_roots = std::collections::VecDeque::with_capacity(validators.len());

        for v in validators.iter() {
            old_delegator_roots.push_back(
                DelegateSmtStorage::generate_sub_proof(
                    &self.smt,
                    v.staker,
                    self.last_checkpoint.epoch,
                    v.delegaters.iter().map(|v| v.0 .0.into()).collect(),
                )
                .await
                .unwrap(),
            )
        }

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
        let mut miner_groups = Vec::with_capacity(validators.len());
        let mut delegator_proofs = Vec::new();
        for v in validators.iter() {
            let delegate_new_epoch_proof = DelegateSmtStorage::generate_sub_proof(
                &self.smt,
                v.staker,
                self.last_checkpoint.epoch + 1,
                v.delegaters.iter().map(|v| v.0 .0.into()).collect(),
            )
            .await
            .unwrap();
            miner_groups.push(MinerGroupInfo {
                staker:               v.staker.0.into(),
                amount:               v.amount,
                delegate_epoch_proof: delegate_new_epoch_proof.clone(),
                delegate_infos:       v
                    .delegaters
                    .iter()
                    .map(|(k, v)| DelegateInfo {
                        delegator_addr: k.0.into(),
                        amount:         *v,
                    })
                    .collect(),
            });
            delegator_proofs.push(DelegateProof {
                staker: v.staker.0.into(),
                proof:  delegate_new_epoch_proof.clone(),
            });
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
                staker: v.staker.0.into(),
                delegate_infos: v
                    .delegaters
                    .iter()
                    .map(|v| DelegateInfo {
                        delegator_addr: v.0 .0.into(),
                        amount:         *v.1,
                    })
                    .collect(),
                delegate_old_epoch_proof: old_delegator_roots.pop_front().unwrap(),
                delegate_new_epoch_proof,
            })
        }

        let xudt = Xudt::type_(&self.type_ids.xudt_owner.pack());
        let new_metadata = {
            let mut new_validators: Vec<Validator> = Vec::with_capacity(validators.len());

            for v in validators {
                let stake_lock = HStake::lock(&self.type_ids.metadata_type_id, &v.staker.0.into());
                let stake_cell = HStake::get_cell(self.ckb, stake_lock, xudt.clone())
                    .await?
                    .expect("Must have stake AT cell");
                let mut stake_data = stake_cell.output_data.unwrap().into_bytes();
                let stake_data = AStakeAtCellData::new_unchecked(stake_data.split_off(TOKEN_BYTES));

                new_validators.push(Validator {
                    bls_pub_key:    stake_data.as_reader().lock().bls_pub_key().to_entity(),
                    pub_key:        stake_data.as_reader().lock().l1_pub_key().to_entity(),
                    address:        v.staker.0.into(),
                    // field not enabled
                    propose_weight: 1,
                    // field not enabled
                    vote_weight:    1,
                    // new epoch start with all zero
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

        // new metadata cell data
        let metadata_cell_data = MetadataCellData {
            metadata: vec![
                last_metadata_cell_data
                    .as_reader()
                    .metadata()
                    .get(1)
                    .unwrap()
                    .to_entity()
                    .into(),
                new_metadata,
            ],

            epoch:                  self.last_checkpoint.epoch + 2,
            propose_count_smt_root: Into::<[u8; 32]>::into(proposal_count_smt_root).into(),
            type_ids:               self.type_ids.clone(),
        };

        let metadata_witness = {
            let stake_smt_election_info = StakeSmtElectionInfo {
                n2:                  ElectionSmtProof {
                    miners:             miner_groups,
                    staker_epoch_proof: new_stake_smt_proof.clone(),
                },
                new_stake_proof:     new_stake_smt_proof,
                new_delegate_proofs: delegator_proofs,
            };

            let witness_data = MetadataWitness {
                new_propose_proof: new_proposal_count_smt_proof,
                smt_election_info: stake_smt_election_info,
            };

            WitnessArgs::new_builder()
                .input_type(Some(Into::<AMetadataWitness>::into(witness_data).as_bytes()).pack())
                .build()
        };
        let stake_smt_cell_data = StakeSmtCellData {
            smt_root:           Into::<[u8; 32]>::into(new_stake_root).into(),
            metadata_type_hash: HMetadata::type_(&metadata_cell_data.type_ids.metadata_type_id)
                .calc_script_hash(),
        };

        let delegate_smt_cell_data = DelegateSmtCellData {
            smt_roots:          delegator_staker_smt_roots,
            metadata_type_hash: HMetadata::type_(&metadata_cell_data.type_ids.metadata_type_id)
                .calc_script_hash(),
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

            let stake_lock = HStake::lock(&self.type_ids.metadata_type_id, &addr.0.into());
            let stake_cell = HStake::get_cell(self.ckb, stake_lock.clone(), xudt.clone())
                .await?
                .expect("Must have stake AT cell");
            let mut stake_data = stake_cell.output_data.unwrap().into_bytes().to_vec();

            no_top_staker_cell_inputs.push(
                CellInput::new_builder()
                    .previous_output(stake_cell.out_point.into())
                    .build(),
            );

            let total_amount = {
                let mut total = [0u8; 16];
                total.copy_from_slice(&stake_data[0..16]);
                u128::from_le_bytes(total) - amount
            };

            for (i, v) in total_amount.to_le_bytes().into_iter().enumerate() {
                stake_data[i] = v
            }

            no_top_staker_cell_outputs.push(
                CellOutput::new_builder()
                    .lock(stake_lock.clone())
                    .type_(Some(xudt.clone()).pack())
                    .build_exact_capacity(Capacity::bytes(stake_data.len())?)?,
            );

            no_top_staker_cell_datas.push(bytes::Bytes::from(stake_data));
            witnesses.push(HStake::witness(1).as_bytes());
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
                            let delegate_lock =
                                HDelegate::lock(&self.type_ids.delegate_code_hash, &addr.0.into());
                            let delegate_cell =
                                HDelegate::get_cell(self.ckb, delegate_lock.clone(), xudt.clone())
                                    .await?
                                    .expect("Must have delegate AT cell");

                            let mut delegate_data = delegate_cell.output_data.unwrap().into_bytes();

                            no_top_delegator_cell_inputs.push(
                                CellInput::new_builder()
                                    .previous_output(delegate_cell.out_point.into())
                                    .build(),
                            );
                            v.insert((
                                CellOutput::new_builder()
                                    .lock(delegate_lock.clone())
                                    .type_(Some(xudt.clone()).pack())
                                    .build_exact_capacity(Capacity::bytes(delegate_data.len())?)?,
                                {
                                    let mut total = [0u8; 16];
                                    total.copy_from_slice(&delegate_data[0..16]);
                                    u128::from_le_bytes(total)
                                },
                                Into::<DelegateAtCellData>::into(
                                    ADelegateAtCellData::new_unchecked(
                                        delegate_data.split_off(TOKEN_BYTES),
                                    ),
                                ),
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

        let (no_top_delegator_cell_outputs, no_top_delegator_cell_output_datas): (
            Vec<CellOutput>,
            Vec<bytes::Bytes>,
        ) = delegator_at_cell_datas
            .into_values()
            .map(|(cell_output, total_amount, data)| {
                witnesses.push(HDelegate::witness(0u8).as_bytes());
                let mut res = total_amount.to_le_bytes().to_vec();
                res.extend((Into::<ADelegateAtCellData>::into(data)).as_slice());
                (cell_output, bytes::Bytes::from(res))
            })
            .unzip();

        let mut withdraw_inputs: Vec<CellInput> = Vec::with_capacity(withdraw_set.len());
        let mut withdraw_outputs: Vec<CellOutput> = Vec::with_capacity(withdraw_set.len());
        let mut withdraw_output_datas: Vec<bytes::Bytes> = Vec::with_capacity(withdraw_set.len());
        for (addr, amount) in withdraw_set {
            let withdraw_lock = Withdraw::lock(&self.type_ids.metadata_type_id, &addr.0.into());
            let withdraw_cell = Withdraw::get_cell(self.ckb, withdraw_lock.clone(), xudt.clone())
                .await?
                .expect("Must have withdraw cell");

            let mut withdraw_data = withdraw_cell.output_data.unwrap().into_bytes();

            withdraw_inputs.push(
                CellInput::new_builder()
                    .previous_output(withdraw_cell.out_point.into())
                    .build(),
            );
            withdraw_outputs.push(
                CellOutput::new_builder()
                    .lock(withdraw_lock)
                    .type_(Some(xudt.clone()).pack())
                    .build_exact_capacity(Capacity::bytes(withdraw_data.len())?)?,
            );
            let total_amount = {
                let mut total = [0u8; 16];
                total.copy_from_slice(&withdraw_data[0..16]);
                u128::from_le_bytes(total) + amount
            };
            let mut withdraw_data = {
                Into::<WithdrawAtCellData>::into(AWithdrawAtCellData::new_unchecked(
                    withdraw_data.split_off(TOKEN_BYTES),
                ))
            };
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
            });
            witnesses.push(Withdraw::witness(true).as_bytes());
        }

        // todo
        let mut outputs_data = vec![
            Into::<AMetadataCellData>::into(metadata_cell_data).as_bytes(),
            Into::<AStakeSmtCellData>::into(stake_smt_cell_data).as_bytes(),
            Into::<ADelegateSmtCellData>::into(delegate_smt_cell_data).as_bytes(),
        ];

        witnesses[0] = metadata_witness.as_bytes();
        witnesses[1] = {
            HStake::smt_witness(
                0,
                stake_smt_update.all_stake_infos,
                stake_smt_update.old_epoch_proof,
                stake_smt_update.new_epoch_proof,
            )
            .as_bytes()
        };
        witnesses[2] = HDelegate::smt_witness(0, delegator_smt_update_infos).as_bytes();

        let mut outputs = vec![
            // metadata cell
            CellOutput::new_builder()
                .lock(AlwaysSuccess::lock())
                .type_(Some(metadata_type).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[0].len())?)?,
            // stake smt cell
            CellOutput::new_builder()
                .lock(AlwaysSuccess::lock())
                .type_(Some(stake_smt).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[1].len())?)?,
            // delegate smt cell
            CellOutput::new_builder()
                .lock(AlwaysSuccess::lock())
                .type_(Some(delegate_smt).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[2].len())?)?,
        ];

        // add no top stakers
        inputs.extend(no_top_staker_cell_inputs);
        outputs.extend(no_top_staker_cell_outputs);
        outputs_data.extend(no_top_staker_cell_datas);

        // add no top delegators
        inputs.extend(no_top_delegator_cell_inputs);
        outputs.extend(no_top_delegator_cell_outputs);
        outputs_data.extend(no_top_delegator_cell_output_datas);

        // add withdraw cells
        inputs.extend(withdraw_inputs);
        outputs.extend(withdraw_outputs);
        outputs_data.extend(withdraw_output_datas);

        // todo
        let cell_deps = vec![
            Secp256k1::lock_dep(),
            OmniEth::lock_dep(),
            AlwaysSuccess::lock_dep(),
            Xudt::type_dep(),
            HDelegate::lock_dep(),
            HDelegate::smt_type_dep(),
            HStake::lock_dep(),
            HStake::smt_type_dep(),
            Withdraw::lock_dep(),
            HCheckpoint::cell_dep(self.ckb, &self.type_ids.checkpoint_type_id).await?,
            HMetadata::cell_dep(self.ckb, &self.type_ids.metadata_type_id).await?,
        ];

        witnesses.push(OmniEth::witness_placeholder().as_bytes()); // capacity provider lock

        // todo: balance tx, fill placeholder witnesses,
        let tx = TransactionBuilder::default()
            .inputs(inputs)
            .outputs(outputs)
            .outputs_data(outputs_data.pack())
            .cell_deps(cell_deps)
            .witnesses(witnesses.pack())
            .build();

        // todo: sign tx

        Ok((tx, no_top_stakers, no_top_delegators))
    }
}
