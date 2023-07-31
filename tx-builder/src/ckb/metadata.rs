use std::{
    collections::HashMap,
    fs::{copy, create_dir_all, remove_file, rename, File},
    io::Write,
    path::PathBuf,
};

use anyhow::Result;
use async_trait::async_trait;
use ckb_sdk::{ScriptGroup, ScriptGroupType};
use ckb_types::{
    core::{Capacity, TransactionBuilder, TransactionView},
    packed::{CellDep, CellInput, CellOutput, WitnessArgs},
    prelude::{Entity, Pack, Reader},
};
use ethereum_types::H160;

use common::traits::{
    ckb_rpc_client::CkbRpc,
    smt::{DelegateSmtStorage, ProposalSmtStorage, StakeSmtStorage},
    tx_builder::IMetadataTxBuilder,
};
use common::types::axon_types::{
    checkpoint::CheckpointCellData,
    delegate::{
        DelegateAtCellData as ADelegateAtCellData, DelegateSmtCellData as ADelegateSmtCellData,
    },
    metadata::{
        MetadataCellData as AMetadataCellData, MetadataList, MetadataWitness as AMetadataWitness,
        ValidatorList,
    },
    stake::{StakeAtCellData as AStakeAtCellData, StakeSmtCellData as AStakeSmtCellData},
};
use common::types::ckb_rpc_client::Cell;
use common::types::tx_builder::*;
use common::utils::convert::{to_byte32, to_uint64};
use molecule::prelude::Builder;

use crate::ckb::define::constants::*;
use crate::ckb::define::types::*;
use crate::ckb::helper::{
    token_cell_data, AlwaysSuccess, Delegate as HDelegate, Metadata as HMetadata, OmniEth,
    Secp256k1, Stake as HStake, Tx, Withdraw, Xudt,
};

const DEFAULT_CONTEXT_PATH: &str = "metadata_context";

pub struct MetadataSmtTxBuilder<'a, C: CkbRpc, PSmt> {
    ckb:                     &'a C,
    kicker:                  PrivateKey,
    type_ids:                MetadataTypeIds,
    last_checkpoint:         Cell,
    smt:                     PSmt,
    last_checkpoint_data:    Checkpoint,
    last_metadata_cell:      Cell,
    last_metadata_cell_data: AMetadataCellData,
    dir:                     PathBuf,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct MetadataContext {
    miner_groups:        Vec<MinerGroupInfo>,
    validators:          Vec<EpochStakeInfo>,
    no_top_stakers:      Vec<(H160, u128)>,
    no_top_delegators:   HashMap<H160, HashMap<H160, u128>>,
    old_stake_smt_proof: Vec<u8>,
    epoch:               Epoch,
}

impl<'a, C: CkbRpc, PSmt> MetadataSmtTxBuilder<'a, C, PSmt>
where
    PSmt: ProposalSmtStorage + StakeSmtStorage + DelegateSmtStorage + Send + 'static + Sync,
{
    async fn generate_context(&self) -> Result<MetadataContext> {
        // load context from file, if it is valid, use it directly and never generate
        if let Some(f) = load_file(&self.dir) {
            match serde_json::from_reader::<_, MetadataContext>(f) {
                Ok(f) => {
                    if f.epoch == self.last_checkpoint_data.epoch {
                        log::info!("[metadata] epoch: {}, load data from file.", f.epoch);
                        return Ok(f);
                    }
                }
                Err(e) => log::debug!("parser metadata error: {}", e),
            }
        }

        let stakers = StakeSmtStorage::get_sub_leaves(
            &self.smt,
            self.last_checkpoint_data.epoch + INAUGURATION,
        )
        .await?;
        let mut miner_groups = Vec::with_capacity(stakers.len());

        let quorum = HMetadata::parse_quorum(&self.last_metadata_cell_data);
        log::info!(
            "[metadta] quorum: {}, stakers count: {}",
            quorum,
            stakers.len(),
        );

        let (validators, no_top_stakers, no_top_delegators) = {
            let mut mid = std::collections::BinaryHeap::new();

            for (staker, amount) in stakers.clone().into_iter() {
                let delegaters = DelegateSmtStorage::get_sub_leaves(
                    &self.smt,
                    self.last_checkpoint_data.epoch + INAUGURATION,
                    staker,
                )
                .await?;

                miner_groups.push(MinerGroupInfo {
                    staker: staker.0.into(),
                    amount,
                    delegate_epoch_proof: DelegateSmtStorage::generate_top_proof(
                        &self.smt,
                        vec![self.last_checkpoint_data.epoch + INAUGURATION],
                        staker,
                    )
                    .await?,
                    delegate_infos: delegaters
                        .iter()
                        .map(|(k, v)| DelegateInfo {
                            delegator_addr: k.0.into(),
                            amount:         *v,
                        })
                        .collect(),
                });

                mid.push(EpochStakeInfo {
                    staker,
                    amount,
                    delegaters,
                });
            }

            let stakers_count = mid.len();

            let stake_infos = {
                let mut res = Vec::with_capacity(quorum as usize);
                for _ in 0..quorum {
                    match mid.pop() {
                        Some(s) => res.push(s),
                        None => break,
                    }
                }
                res
            };

            if stakers_count <= quorum as usize {
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

        let old_stake_smt_proof = StakeSmtStorage::generate_top_proof(&self.smt, vec![
            self.last_checkpoint_data.epoch + INAUGURATION,
        ])
        .await
        .unwrap();

        let context = MetadataContext {
            miner_groups,
            validators,
            no_top_stakers,
            no_top_delegators,
            old_stake_smt_proof,
            epoch: self.last_checkpoint_data.epoch,
        };

        dump_to_dir(&context, &self.dir);

        Ok(context)
    }

    async fn generate_staker(
        &self,
        context: &MetadataContext,
    ) -> Result<(Vec<u8>, StakeSmtCellData)> {
        StakeSmtStorage::remove(
            &self.smt,
            self.last_checkpoint_data.epoch + INAUGURATION,
            context
                .no_top_stakers
                .iter()
                .map(|(a, _)| a)
                .cloned()
                .collect(),
        )
        .await
        .unwrap();

        StakeSmtStorage::new_epoch(
            &self.smt,
            self.last_checkpoint_data.epoch + INAUGURATION + 1,
        )
        .await
        .unwrap();

        let new_stake_smt_proof = StakeSmtStorage::generate_top_proof(&self.smt, vec![
            self.last_checkpoint_data.epoch + INAUGURATION + 1,
        ])
        .await
        .unwrap();

        let new_stake_root = StakeSmtStorage::get_top_root(&self.smt).await.unwrap();

        let stake_smt_cell_data = StakeSmtCellData {
            smt_root:           Into::<[u8; 32]>::into(new_stake_root).into(),
            metadata_type_hash: HMetadata::type_(&self.type_ids.metadata_type_id)
                .calc_script_hash(),
        };

        Ok((new_stake_smt_proof, stake_smt_cell_data))
    }

    async fn generate_delegator(
        &self,
        context: &MetadataContext,
    ) -> Result<(Vec<DelegateProof>, DelegateSmtCellData)> {
        let delegatets_remove_keys = context
            .no_top_delegators
            .iter()
            .flat_map(|(d, i)| i.keys().cloned().zip(std::iter::repeat(*d)))
            .collect();

        DelegateSmtStorage::remove(
            &self.smt,
            self.last_checkpoint_data.epoch + INAUGURATION,
            delegatets_remove_keys,
        )
        .await
        .unwrap();

        DelegateSmtStorage::new_epoch(
            &self.smt,
            self.last_checkpoint_data.epoch + INAUGURATION + 1,
        )
        .await
        .unwrap();

        let mut delegator_staker_smt_roots = Vec::with_capacity(context.validators.len());
        let mut new_delegator_proofs = Vec::new();
        for v in context.validators.iter() {
            log::info!(
                "[metadata] validator: {}, amount: {}",
                v.staker.to_string(),
                v.amount,
            );

            let delegate_new_epoch_proof = DelegateSmtStorage::generate_top_proof(
                &self.smt,
                vec![self.last_checkpoint_data.epoch + INAUGURATION + 1],
                v.staker,
            )
            .await
            .unwrap();
            new_delegator_proofs.push(DelegateProof {
                staker: v.staker.0.into(),
                proof:  delegate_new_epoch_proof.clone(),
            });
            delegator_staker_smt_roots.push(StakerSmtRoot {
                staker: v.staker.0.into(),
                root:   Into::<[u8; 32]>::into(
                    DelegateSmtStorage::get_top_root(&self.smt, v.staker)
                        .await
                        .unwrap(),
                )
                .into(),
            });
        }
        let delegate_smt_cell_data = DelegateSmtCellData {
            smt_roots:          delegator_staker_smt_roots,
            metadata_type_hash: HMetadata::type_(&self.type_ids.metadata_type_id)
                .calc_script_hash(),
        };

        Ok((new_delegator_proofs, delegate_smt_cell_data))
    }

    async fn generate_metadata(
        &self,
        context: &MetadataContext,
        new_stake_smt_proof: Vec<u8>,
        new_delegator_proofs: Vec<DelegateProof>,
    ) -> Result<(AMetadataCellData, WitnessArgs)> {
        let xudt = Xudt::type_(&self.type_ids.xudt_owner.pack());
        ProposalSmtStorage::insert(
            &self.smt,
            self.last_checkpoint_data.epoch,
            self.last_checkpoint_data
                .propose_count
                .iter()
                .map(|v| (v.proposer.0.into(), v.count))
                .collect(),
        )
        .await?;

        let proposal_count_smt_root = ProposalSmtStorage::get_top_root(&self.smt).await?;
        let new_proposal_count_smt_proof = ProposalSmtStorage::generate_top_proof(&self.smt, vec![
            self.last_checkpoint_data.epoch,
        ])
        .await
        .unwrap();

        let new_metadata = {
            let mut new_validators = Vec::new();

            for v in context.validators.iter() {
                let stake_lock = HStake::lock(&self.type_ids.metadata_type_id, &v.staker.0.into());
                let stake_cell = HStake::get_cell(self.ckb, stake_lock, xudt.clone())
                    .await?
                    .expect("Must have stake AT cell");
                let mut stake_data = stake_cell.output_data.unwrap().into_bytes();
                let stake_data = AStakeAtCellData::new_unchecked(stake_data.split_off(TOKEN_BYTES));

                new_validators.push(Validator {
                    bls_pub_key:    stake_data
                        .as_reader()
                        .lock()
                        .bls_pub_key()
                        .to_entity()
                        .as_bytes(),
                    pub_key:        stake_data
                        .as_reader()
                        .lock()
                        .l1_pub_key()
                        .to_entity()
                        .as_bytes(),
                    address:        v.staker.0.into(),
                    // field not enabled
                    propose_weight: 1,
                    // field not enabled
                    vote_weight:    1,
                    // new epoch start with all zero
                    propose_count:  0,
                });
            }

            self.last_metadata_cell_data
                .metadata()
                .get(1)
                .unwrap()
                .as_builder()
                .validators({
                    new_validators.sort();
                    let mut validators = ValidatorList::new_builder();
                    for v in new_validators.into_iter() {
                        validators = validators.push(v.into());
                    }
                    validators.build()
                })
                .build()
        };

        // new metadata cell data
        let metadata_cell_data = {
            let next_metadata = self
                .last_metadata_cell_data
                .as_reader()
                .metadata()
                .get(1)
                .unwrap()
                .to_entity();
            self.last_metadata_cell_data
                .clone()
                .as_builder()
                .epoch(to_uint64(self.last_checkpoint_data.epoch + 1))
                .propose_count_smt_root(to_byte32(
                    &Into::<[u8; 32]>::into(proposal_count_smt_root).into(),
                ))
                .metadata({
                    let mut list = MetadataList::new_builder();
                    list = list.push(next_metadata);
                    list = list.push(new_metadata);
                    list.build()
                })
                .build()
        };

        let metadata_witness = {
            let stake_smt_election_info = StakeSmtElectionInfo {
                n2:                  ElectionSmtProof {
                    miners:             context.miner_groups.clone(),
                    staker_epoch_proof: context.old_stake_smt_proof.clone(),
                },
                new_stake_proof:     new_stake_smt_proof,
                new_delegate_proofs: new_delegator_proofs,
            };

            let witness_data = MetadataWitness {
                new_propose_proof: new_proposal_count_smt_proof,
                smt_election_info: stake_smt_election_info,
            };

            WitnessArgs::new_builder()
                .input_type(Some(Into::<AMetadataWitness>::into(witness_data).as_bytes()).pack())
                .build()
        };

        Ok((metadata_cell_data, metadata_witness))
    }

    async fn no_top_process(
        &self,
        context: &MetadataContext,
    ) -> Result<(
        Vec<bytes::Bytes>,
        Vec<CellInput>,
        Vec<CellOutput>,
        Vec<bytes::Bytes>,
    )> {
        let xudt = Xudt::type_(&self.type_ids.xudt_owner.pack());
        let mut witnesses = Vec::new();
        let mut withdraw_set: HashMap<H160, u128> = HashMap::default();
        // remove no top staker
        let mut no_top_staker_cell_datas = Vec::with_capacity(context.no_top_stakers.len());
        let mut no_top_staker_cell_inputs: Vec<CellInput> =
            Vec::with_capacity(context.no_top_stakers.len());
        let mut no_top_staker_cell_outputs: Vec<CellOutput> =
            Vec::with_capacity(context.no_top_stakers.len());

        for (addr, amount) in context.no_top_stakers.iter() {
            *withdraw_set.entry(*addr).or_default() += amount;

            let stake_lock = HStake::lock(&self.type_ids.metadata_type_id, &addr.0.into());
            let stake_cell = HStake::get_cell(self.ckb, stake_lock.clone(), xudt.clone())
                .await?
                .expect("Must have stake AT cell");

            let (total_amount, stake_data) = HStake::parse_stake_data(&stake_cell);

            no_top_staker_cell_inputs.push(
                CellInput::new_builder()
                    .previous_output(stake_cell.out_point.into())
                    .build(),
            );

            let stake_data = token_cell_data(total_amount - amount, stake_data.as_bytes());
            log::info!(
                "[metadata] none top staker: {}, smt amount: {}, old total stake amount: {}, withdraw amount: {}",
                addr.to_string(), amount, total_amount, amount,
            );

            no_top_staker_cell_outputs.push(
                CellOutput::new_builder()
                    .lock(stake_lock.clone())
                    .type_(Some(xudt.clone()).pack())
                    .build_exact_capacity(Capacity::bytes(stake_data.len())?)?,
            );

            no_top_staker_cell_datas.push(stake_data);
            witnesses.push(HStake::witness(1).as_bytes());
        }

        // remove no top delegator
        let mut no_top_delegator_cell_inputs: Vec<CellInput> =
            Vec::with_capacity(context.no_top_stakers.len());
        let mut delegator_at_cell_datas = HashMap::with_capacity(context.no_top_delegators.len());
        for (staker_address, v) in context.no_top_delegators.iter() {
            log::info!(
                "[metadata] staker: {}, none delegators: ",
                staker_address.to_string()
            );
            for (addr, amount) in v {
                *withdraw_set.entry(*addr).or_default() += amount;

                let (_cell_output, total_amount, delegator_at_cell_data) =
                    match delegator_at_cell_datas.entry(*addr) {
                        std::collections::hash_map::Entry::Occupied(v) => v.into_mut(),
                        std::collections::hash_map::Entry::Vacant(v) => {
                            let delegate_lock =
                                HDelegate::lock(&self.type_ids.metadata_type_id, &addr.0.into());
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

                log::info!(
                    "[metadata] none top delegator: {}, smt amount: {}, old total stake amount: {}, withdraw amount: {}",
                    addr.to_string(), amount, total_amount, amount,
                );
            }
        }

        let (no_top_delegator_cell_outputs, no_top_delegator_cell_output_datas): (
            Vec<CellOutput>,
            Vec<bytes::Bytes>,
        ) = delegator_at_cell_datas
            .into_values()
            .map(|(cell_output, total_amount, data)| {
                witnesses.push(HDelegate::witness(1u8).as_bytes());
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

            let withdraw_data =
                Withdraw::update_cell_data(&withdraw_cell, self.last_checkpoint_data.epoch, amount);

            withdraw_inputs.push(
                CellInput::new_builder()
                    .previous_output(withdraw_cell.out_point.clone().into())
                    .build(),
            );

            withdraw_outputs.push(
                CellOutput::new_builder()
                    .lock(withdraw_lock)
                    .type_(Some(xudt.clone()).pack())
                    .build_exact_capacity(Capacity::bytes(withdraw_data.len())?)?,
            );

            withdraw_output_datas.push(withdraw_data);
            witnesses.push(Withdraw::witness(true).as_bytes());
        }

        let inputs = no_top_staker_cell_inputs
            .into_iter()
            .chain(no_top_delegator_cell_inputs.into_iter())
            .chain(withdraw_inputs.into_iter())
            .collect();

        let outputs = no_top_staker_cell_outputs
            .into_iter()
            .chain(no_top_delegator_cell_outputs.into_iter())
            .chain(withdraw_outputs.into_iter())
            .collect();

        let output_datas = no_top_staker_cell_datas
            .into_iter()
            .chain(no_top_delegator_cell_output_datas.into_iter())
            .chain(withdraw_output_datas.into_iter())
            .collect();

        Ok((witnesses, inputs, outputs, output_datas))
    }

    async fn build_tx(self) -> Result<TransactionView> {
        let context = self.generate_context().await?;
        let (new_stake_smt_proof, stake_smt_cell_data) = self.generate_staker(&context).await?;
        let (new_delegator_proofs, delegate_smt_cell_data) =
            self.generate_delegator(&context).await?;
        let (metadata_cell_data, metadata_witness) = self
            .generate_metadata(&context, new_stake_smt_proof, new_delegator_proofs)
            .await?;
        let (no_top_witnesses, no_top_inputs, no_top_outputs, no_top_output_datas) =
            self.no_top_process(&context).await?;

        let metadata_type = HMetadata::type_(&self.type_ids.metadata_type_id);

        let stake_smt = HStake::smt_type(&self.type_ids.stake_smt_type_id);

        let last_stake_smt_cell = HStake::get_smt_cell(self.ckb, stake_smt.clone()).await?;

        let delegate_smt = HDelegate::smt_type(&self.type_ids.delegate_smt_type_id);

        let last_delegate_smt_cell =
            HDelegate::get_smt_cell(self.ckb, delegate_smt.clone()).await?;

        let mut inputs = vec![
            // metadata
            CellInput::new_builder()
                .previous_output(self.last_metadata_cell.out_point.into())
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
            // metadata type
            metadata_witness.as_bytes(),
            // stake smt type
            HStake::smt_witness(1, vec![], vec![], vec![]).as_bytes(),
            // delegate smt type
            HDelegate::smt_witness(1, vec![]).as_bytes(),
        ];

        let mut outputs_data = vec![
            Into::<AMetadataCellData>::into(metadata_cell_data).as_bytes(),
            Into::<AStakeSmtCellData>::into(stake_smt_cell_data).as_bytes(),
            Into::<ADelegateSmtCellData>::into(delegate_smt_cell_data).as_bytes(),
        ];

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

        witnesses.extend(no_top_witnesses);
        inputs.extend(no_top_inputs);
        outputs.extend(no_top_outputs);
        outputs_data.extend(no_top_output_datas);

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
            HMetadata::type_dep(),
            // checkpoint cell dep
            CellDep::new_builder()
                .out_point(self.last_checkpoint.out_point.into())
                .build(),
        ];

        witnesses.push(OmniEth::witness_placeholder().as_bytes()); // capacity provider lock

        let tx = TransactionBuilder::default()
            .inputs(inputs)
            .outputs(outputs)
            .outputs_data(outputs_data.pack())
            .cell_deps(cell_deps)
            .witnesses(witnesses.pack())
            .build();

        let omni_eth = OmniEth::new(self.kicker.clone());
        let kicker_lock = OmniEth::lock(&omni_eth.address()?);

        let mut tx = Tx::new(self.ckb, tx);
        tx.balance(kicker_lock.clone()).await?;

        tx.sign(&omni_eth.signer()?, &ScriptGroup {
            script:         kicker_lock,
            group_type:     ScriptGroupType::Lock,
            input_indices:  vec![tx.inner_ref().witnesses().len() - 1],
            output_indices: vec![],
        })?;

        Ok(tx.inner())
    }
}

#[async_trait]
impl<'a, C: CkbRpc, PSmt> IMetadataTxBuilder<'a, C, PSmt> for MetadataSmtTxBuilder<'a, C, PSmt>
where
    PSmt: ProposalSmtStorage + StakeSmtStorage + DelegateSmtStorage + Send + 'static + Sync,
{
    async fn new(
        ckb: &'a C,
        kicker: PrivateKey,
        type_ids: MetadataTypeIds,
        last_checkpoint: Cell,
        smt: PSmt,
        dir: PathBuf,
    ) -> Self {
        let checkpoint_data = last_checkpoint.output_data.clone().unwrap().into_bytes();
        let checkpoint_data = CheckpointCellData::new_unchecked(checkpoint_data);
        let last_checkpoint_data: Checkpoint = checkpoint_data.into();

        let metadata_type = HMetadata::type_(&type_ids.metadata_type_id);

        let last_metadata_cell = HMetadata::get_cell(ckb, metadata_type.clone())
            .await
            .unwrap();

        let last_metadata_cell_data = AMetadataCellData::new_unchecked(
            last_metadata_cell.output_data.clone().unwrap().into_bytes(),
        );
        Self {
            ckb,
            kicker,
            type_ids,
            last_checkpoint,
            last_checkpoint_data,
            smt,
            last_metadata_cell,
            last_metadata_cell_data,
            dir,
        }
    }

    async fn build_tx(self) -> Result<TransactionView> {
        self.build_tx().await
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
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

fn load_file(dir: &PathBuf) -> Option<std::io::BufReader<File>> {
    create_dir_all(dir).unwrap();
    let path = dir.join(DEFAULT_CONTEXT_PATH);

    File::open(path).map(std::io::BufReader::new).ok()
}

fn dump_to_dir(context: &MetadataContext, dir: &PathBuf) {
    create_dir_all(dir).unwrap();
    let tmp_dir = dir.join("tmp");
    create_dir_all(&tmp_dir).unwrap();

    let tmp_file = tmp_dir.join(DEFAULT_CONTEXT_PATH);
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(false)
        .open(&tmp_file)
        .unwrap();
    file.set_len(0)
        .and_then(|_| serde_json::to_string(&context).map_err(Into::into))
        .and_then(|json_string| file.write_all(json_string.as_bytes()))
        .and_then(|_| file.sync_all())
        .unwrap();
    move_file(tmp_file, dir.join(DEFAULT_CONTEXT_PATH));
}

fn move_file<P: AsRef<std::path::Path>>(src: P, dst: P) {
    if rename(&src, &dst).is_err() {
        copy(&src, &dst).unwrap();
        remove_file(&src).unwrap();
    }
}
