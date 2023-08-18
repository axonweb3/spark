use std::cmp::min;
use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use ckb_types::{
    bytes::Bytes,
    core::{Capacity, TransactionBuilder, TransactionView},
    packed::{CellDep, CellInput, CellOutput, WitnessArgs},
    prelude::{Entity, Pack},
    H160,
};
use molecule::prelude::Builder;

use common::{
    traits::ckb_rpc_client::CkbRpc,
    traits::smt::{DelegateSmtStorage, ProposalSmtStorage, RewardSmtStorage, StakeSmtStorage},
    traits::tx_builder::IRewardTxBuilder,
    types::axon_types::{
        delegate::DelegateCellData,
        metadata::MetadataCellData,
        reward::{RewardSmtCellData as ARewardSmtCellData, RewardWitness as ARewardWitness},
    },
    types::tx_builder::{Amount, Epoch, EthAddress, RewardMeta, RewardTypeIds},
    utils::convert::{to_ckb_h160, to_eth_h160},
};

use crate::ckb::define::constants::{INAUGURATION, START_EPOCH};
use crate::ckb::define::error::CkbTxErr;
use crate::ckb::define::types::{
    EpochRewardStakeInfo, NotClaimInfo, RewardDelegateInfo, RewardSmtCellData, RewardStakeInfo,
    RewardWitness,
};
use crate::ckb::helper::{
    AlwaysSuccess, Checkpoint, Delegate, Metadata, OmniEth, Reward, Secp256k1, Selection, Stake,
    Tx, Xudt,
};

pub struct RewardTxBuilder<'a, C, S>
where
    C: CkbRpc,
    S: RewardSmtStorage + StakeSmtStorage + DelegateSmtStorage + ProposalSmtStorage,
{
    ckb:                   &'a C,
    type_ids:              RewardTypeIds,
    reward_meta:           RewardMeta,
    smt:                   S,
    user:                  EthAddress,
    current_epoch:         Epoch,
    epoch_count:           u64,
    metadata_outpoint:     ckb_jsonrpc_types::OutPoint,
    minimum_propose_count: u64,
    commission_rates:      HashMap<H160, u8>,
    stake_cell_deps:       Vec<CellDep>,
    requirement_cell_deps: Vec<CellDep>,
}

#[async_trait]
impl<'a, C, S> IRewardTxBuilder<'a, C, S> for RewardTxBuilder<'a, C, S>
where
    C: CkbRpc,
    S: RewardSmtStorage + StakeSmtStorage + DelegateSmtStorage + ProposalSmtStorage,
{
    async fn new(
        ckb: &'a C,
        type_ids: RewardTypeIds,
        smt: S,
        user: EthAddress,
        current_epoch: Epoch,
        epoch_count: u64,
    ) -> Self {
        let metadata_cell = Metadata::get_cell(ckb, Metadata::type_(&type_ids.metadata_type_id))
            .await
            .expect("Metadata cell not found");
        let metadata_cell_data = MetadataCellData::new_unchecked(
            metadata_cell.output_data.clone().unwrap().into_bytes(),
        );

        let minimum_propose_count = Metadata::calc_minimum_propose_count(&metadata_cell_data);
        log::info!("[reward] minimum propose count: {}", minimum_propose_count);

        let reward_metadata = Metadata::parse_reward_meta(&metadata_cell_data);
        log::info!("[reward] reward metadata: {:?}", reward_metadata);

        Self {
            ckb,
            type_ids,
            reward_meta: Metadata::parse_reward_meta(&metadata_cell_data),
            smt,
            user,
            current_epoch,
            epoch_count,
            metadata_outpoint: metadata_cell.out_point,
            minimum_propose_count,
            commission_rates: HashMap::new(),
            stake_cell_deps: Vec::new(),
            requirement_cell_deps: Vec::new(),
        }
    }

    async fn build_tx(mut self) -> Result<TransactionView> {
        if self.current_epoch < 4 {
            return Err(CkbTxErr::RewardCurrentEpoch(self.current_epoch).into());
        }

        let reward_smt_cell = Reward::get_cell(self.ckb, &self.type_ids.reward_smt_type_id).await?;
        let selection_cell =
            Selection::get_cell(self.ckb, &self.type_ids.selection_type_id).await?;

        let mut inputs = vec![
            // reward smt cell
            CellInput::new_builder()
                .previous_output(reward_smt_cell.out_point.into())
                .build(),
            // selection cell
            CellInput::new_builder()
                .previous_output(selection_cell.out_point.into())
                .build(),
        ];

        // AT cell
        let token_amount = self.add_token_to_inputs(&mut inputs).await?;

        // 1. Build outputs data.
        // 2. Add each staker's stake AT cell and delegate requrement cell dep.
        // 3. Build witness for reward smt cell.
        let (outputs_data, reward_witness) = self
            .build_data_and_witness(token_amount.unwrap_or(0))
            .await?;

        let outputs = vec![
            // reward smt cell
            CellOutput::new_builder()
                .lock(AlwaysSuccess::lock())
                .type_(Some(Reward::smt_type(&self.type_ids.reward_smt_type_id)).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[0].len())?)?,
            // selection cell
            CellOutput::new_builder()
                .lock(AlwaysSuccess::lock())
                .type_(Some(Selection::type_(&self.type_ids.selection_type_id)).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[1].len())?)?,
            // AT cell
            CellOutput::new_builder()
                .lock(OmniEth::lock(&self.user))
                .type_(Some(Xudt::type_(&self.type_ids.xudt_owner.pack())).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[2].len())?)?,
        ];

        let mut cell_deps = vec![
            OmniEth::lock_dep(),
            Secp256k1::lock_dep(),
            AlwaysSuccess::lock_dep(),
            Xudt::type_dep(),
            Reward::smt_type_dep(),
            Selection::lock_dep(),
            Checkpoint::cell_dep(self.ckb, &self.type_ids.checkpoint_type_id).await?,
            Stake::smt_cell_dep(self.ckb, &self.type_ids.stake_smt_type_id).await?,
            Delegate::smt_cell_dep(self.ckb, &self.type_ids.delegate_smt_type_id).await?,
            // metadata cell dep
            CellDep::new_builder()
                .out_point(self.metadata_outpoint.clone().into())
                .build(),
        ];
        cell_deps.extend(self.stake_cell_deps);
        cell_deps.extend(self.requirement_cell_deps);

        let mut witnesses = vec![
            WitnessArgs::new_builder()
                .input_type(Some(ARewardWitness::from(reward_witness).as_bytes()).pack())
                .build()
                .as_bytes(),
            bytes::Bytes::default(), // selection cell lock & type
        ];
        if token_amount.is_some() {
            witnesses.push(OmniEth::witness_placeholder().as_bytes()); // AT cell lock
        }
        witnesses.push(OmniEth::witness_placeholder().as_bytes()); // capacity provider lock

        let tx = TransactionBuilder::default()
            .inputs(inputs)
            .outputs(outputs)
            .outputs_data(outputs_data.pack())
            .cell_deps(cell_deps)
            .witnesses(witnesses.pack())
            .build();

        let mut tx = Tx::new(self.ckb, tx);
        tx.balance(OmniEth::lock(&self.user)).await?;

        Ok(tx.inner())
    }
}

impl<'a, C, S> RewardTxBuilder<'a, C, S>
where
    C: CkbRpc,
    S: RewardSmtStorage + StakeSmtStorage + DelegateSmtStorage + ProposalSmtStorage,
{
    async fn add_token_to_inputs(&self, inputs: &mut Vec<CellInput>) -> Result<Option<Amount>> {
        let (token_cells, amount) = Xudt::collect(
            self.ckb,
            OmniEth::lock(&self.user),
            Xudt::type_(&self.type_ids.xudt_owner.pack()),
            1,
        )
        .await?;

        if token_cells.is_empty() {
            return Ok(None);
        }

        // AT cell
        inputs.push(
            CellInput::new_builder()
                .previous_output(token_cells[0].out_point.clone().into())
                .build(),
        );

        Ok(Some(amount))
    }

    async fn build_data_and_witness(
        &mut self,
        mut wallet_amount: Amount,
    ) -> Result<(Vec<Bytes>, RewardWitness)> {
        log::info!(
            "[reward] user: {}, old wallet amount: {}",
            self.user.to_string(),
            wallet_amount,
        );

        if self.current_epoch < INAUGURATION {
            return Err(CkbTxErr::EpochTooSmall.into());
        }

        let mut witness = RewardWitness {
            miner: self.user.clone(),
            ..Default::default()
        };

        let start_reward_epoch = self.get_start_epoch(&mut witness).await?;
        let end_reward_epoch = min(
            self.current_epoch - INAUGURATION,
            start_reward_epoch + self.epoch_count - 1,
        );

        if start_reward_epoch < end_reward_epoch {
            return Err(CkbTxErr::RewardEpoch(start_reward_epoch, end_reward_epoch).into());
        }

        log::info!(
            "[reward] start epoch: {}, end epoch: {}",
            start_reward_epoch,
            end_reward_epoch,
        );

        let mut total_reward_amount = 0_u128;
        let user = to_eth_h160(&self.user);

        for epoch in start_reward_epoch..=end_reward_epoch {
            let propose_counts = ProposalSmtStorage::get_sub_leaves(&self.smt, epoch)
                .await
                .unwrap();

            let mut epoch_reward_witness = EpochRewardStakeInfo::default();
            let mut validators = vec![];

            total_reward_amount += self
                .calc_epoch_reward(
                    user,
                    epoch,
                    propose_counts,
                    &mut epoch_reward_witness,
                    &mut validators,
                )
                .await?;

            epoch_reward_witness.count_proof =
                ProposalSmtStorage::generate_sub_proof(&self.smt, epoch, validators.clone())
                    .await
                    .unwrap();
            epoch_reward_witness.count_root = ProposalSmtStorage::get_sub_root(&self.smt, epoch)
                .await
                .unwrap()
                .unwrap();
            epoch_reward_witness.count_epoch_proof =
                ProposalSmtStorage::generate_top_proof(&self.smt, vec![epoch])
                    .await
                    .unwrap();
            epoch_reward_witness.amount_proof =
                StakeSmtStorage::generate_sub_proof(&self.smt, epoch, validators)
                    .await
                    .unwrap();
            epoch_reward_witness.amount_root = StakeSmtStorage::get_sub_root(&self.smt, epoch)
                .await
                .unwrap()
                .unwrap();
            epoch_reward_witness.amount_epoch_proof =
                StakeSmtStorage::generate_top_proof(&self.smt, vec![epoch])
                    .await
                    .unwrap();

            witness.reward_infos.push(epoch_reward_witness);
        }

        wallet_amount += total_reward_amount;

        log::info!(
            "[reward] user: {}, new wallet amount: {}",
            self.user.to_string(),
            wallet_amount,
        );

        RewardSmtStorage::insert(&self.smt, end_reward_epoch + 1, user).await?;
        witness.new_not_claim_info = NotClaimInfo {
            epoch: end_reward_epoch + 1,
            proof: RewardSmtStorage::generate_proof(&self.smt, vec![to_eth_h160(&self.user)])
                .await
                .unwrap(),
        };

        let reward_smt_root = RewardSmtStorage::get_root(&self.smt).await?;

        Ok((
            vec![
                // reward smt cell data
                ARewardSmtCellData::from(RewardSmtCellData {
                    claim_smt_root:     reward_smt_root,
                    metadata_type_hash: Metadata::type_(&self.type_ids.metadata_type_id)
                        .calc_script_hash(),
                })
                .as_bytes(),
                // selection cell data
                Bytes::default(),
                // AT cell data
                wallet_amount.pack().as_bytes(),
            ],
            witness,
        ))
    }

    async fn get_start_epoch(&self, witness: &mut RewardWitness) -> Result<Epoch> {
        let start_reward_epoch = RewardSmtStorage::get_epoch(&self.smt, to_eth_h160(&self.user))
            .await
            .unwrap();

        if start_reward_epoch.is_none() {
            witness.old_not_claim_info = NotClaimInfo {
                epoch: START_EPOCH + INAUGURATION,
                proof: RewardSmtStorage::generate_proof(&self.smt, vec![to_eth_h160(&self.user)])
                    .await
                    .unwrap(),
            };
            return Ok(START_EPOCH + INAUGURATION);
        }

        witness.old_not_claim_info = NotClaimInfo {
            epoch: start_reward_epoch.unwrap(),
            proof: RewardSmtStorage::generate_proof(&self.smt, vec![to_eth_h160(&self.user)])
                .await
                .unwrap(),
        };

        Ok(start_reward_epoch.unwrap())
    }

    async fn commission_rate(&mut self, staker: &H160) -> Result<u8> {
        if self.commission_rates.contains_key(staker) {
            return Ok(*self.commission_rates.get(staker).unwrap());
        }

        let (requirement_type_id, stake_cell_outpoint) = Stake::get_delegate_requirement_type_id(
            self.ckb,
            &self.type_ids.metadata_type_id,
            staker,
            &self.type_ids.xudt_owner,
        )
        .await?;

        let delegate_requirement_cell = Delegate::get_requirement_cell(
            self.ckb,
            Delegate::requirement_type(&self.type_ids.metadata_type_id, &requirement_type_id),
        )
        .await?;

        self.stake_cell_deps.push(
            CellDep::new_builder()
                .out_point(delegate_requirement_cell.out_point.clone().into())
                .build(),
        );

        self.requirement_cell_deps.push(
            CellDep::new_builder()
                .out_point(stake_cell_outpoint.into())
                .build(),
        );

        let data = delegate_requirement_cell.output_data.unwrap().into_bytes();
        let requirement_cell_data = DelegateCellData::new_unchecked(data);

        Ok(requirement_cell_data
            .delegate_requirement()
            .commission_rate()
            .into())
    }

    async fn calc_epoch_reward(
        &mut self,
        user: ethereum_types::H160,
        epoch: u64,
        propose_counts: HashMap<ethereum_types::H160, u64>,
        epoch_reward_witness: &mut EpochRewardStakeInfo,
        validators: &mut Vec<ethereum_types::H160>,
    ) -> Result<u128> {
        let mut epoch_reward = 0;

        for (validator, propose_count) in propose_counts.into_iter() {
            validators.push(validator);

            let is_validator = user == validator;

            let delegate_amount =
                DelegateSmtStorage::get_amount(&self.smt, epoch + INAUGURATION, validator, user)
                    .await
                    .unwrap();

            let in_delegate_smt = delegate_amount.is_some();

            let commission_rate = self.commission_rate(&to_ckb_h160(&validator)).await? as u128;

            log::info!(
                "[reward] epoch: {}, validator: {}, commission_rate: {}, propose count: {}",
                epoch,
                validator.to_string(),
                commission_rate,
                propose_count,
            );

            let mut total_reward = self.reward_meta.base_reward
                / (2_u64.pow((self.current_epoch / self.reward_meta.half_reward_cycle) as u32))
                    as u128;

            if propose_count < self.minimum_propose_count {
                total_reward = total_reward * self.reward_meta.propose_discount_rate as u128 / 100;
            }

            let stake_amount = StakeSmtStorage::get_amount(&self.smt, epoch, validator).await?;
            if stake_amount.is_none() {
                return Err(CkbTxErr::StakeAmountNotFound(epoch, validator).into());
            }
            let stake_amount = stake_amount.unwrap();

            let all_delegates =
                DelegateSmtStorage::get_sub_leaves(&self.smt, epoch, validator).await?;
            let total_delegate_amount = all_delegates.values().sum::<Amount>();

            let total_amount = stake_amount + total_delegate_amount;
            let staker_reward = total_reward * stake_amount / total_amount;
            let delegators_reward = total_reward - staker_reward;

            log::info!(
                "[reward] epoch: {}, stake amount: {}, total delegate amount: {}, total reward: {}, staker reward: {}, delegators reward: {}",
                epoch, stake_amount, total_delegate_amount, total_reward, staker_reward, delegators_reward,
            );

            if is_validator {
                let staker_fee_reward = delegators_reward * commission_rate / 100;
                epoch_reward += staker_reward + staker_fee_reward;
                log::info!(
                    "[reward] epoch: {}, is a validator, reward: {}",
                    epoch,
                    staker_reward + staker_fee_reward,
                );
            } else if in_delegate_smt {
                let delegate_reward = delegators_reward * delegate_amount.unwrap()
                    / total_delegate_amount
                    * (100 - commission_rate)
                    / 100;
                epoch_reward += delegate_reward;
                log::info!(
                    "[reward] epoch: {}, delegate amount: {}, reward: {}",
                    epoch,
                    delegate_amount.unwrap(),
                    delegate_reward,
                );
            }

            log::info!(
                "[reward] validator: {:?}, propose count: {}, stake amount: {}, delegators count: {}",
                validator.to_string(), propose_count, stake_amount, all_delegates.len()
            );

            epoch_reward_witness
                .reward_stake_infos
                .push(RewardStakeInfo {
                    validator: to_ckb_h160(&validator),
                    propose_count,
                    stake_amount,
                    delegate_infos: all_delegates
                        .into_iter()
                        .map(|(delegator_addr, amount)| RewardDelegateInfo {
                            delegator_addr,
                            amount,
                        })
                        .collect(),
                    delegate_epoch_proof: DelegateSmtStorage::generate_top_proof(
                        &self.smt,
                        vec![epoch],
                        validator,
                    )
                    .await
                    .unwrap(),
                });
        }
        Ok(epoch_reward)
    }
}
