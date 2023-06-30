use anyhow::Result;
use async_trait::async_trait;
use ckb_types::{
    bytes::Bytes,
    core::{Capacity, TransactionBuilder, TransactionView},
    packed::{CellDep, CellInput, CellOutput, Script},
    prelude::{Entity, Pack},
    H160,
};
use molecule::prelude::Builder;

use common::traits::smt::{
    DelegateSmtStorage, ProposalSmtStorage, RewardSmtStorage, StakeSmtStorage,
};
use common::traits::tx_builder::IRewardTxBuilder;
use common::types::axon_types::{
    basic::Byte32, delegate::DelegateRequirement, reward::RewardSmtCellData,
};
use common::types::tx_builder::{Amount, Epoch, EthAddress, RewardInfo, RewardTypeIds};
use common::{
    traits::ckb_rpc_client::CkbRpc,
    utils::convert::{to_ckb_h160, to_eth_h160},
};

use crate::ckb::define::constants::INAUGURATION;
use crate::ckb::define::error::CkbTxErr;
use crate::ckb::helper::{
    AlwaysSuccess, Checkpoint, Delegate, Metadata, OmniEth, Reward, Secp256k1, Stake, Tx, Xudt,
};

pub struct RewardTxBuilder<'a, C, S>
where
    C: CkbRpc,
    S: RewardSmtStorage + StakeSmtStorage + DelegateSmtStorage + ProposalSmtStorage,
{
    ckb:           &'a C,
    type_ids:      RewardTypeIds,
    smt:           S,
    info:          RewardInfo,
    user:          EthAddress,
    current_epoch: Epoch,
    token_lock:    Script,
    xudt:          Script,
}

#[async_trait]
impl<'a, C, S> IRewardTxBuilder<'a, C, S> for RewardTxBuilder<'a, C, S>
where
    C: CkbRpc,
    S: RewardSmtStorage + StakeSmtStorage + DelegateSmtStorage + ProposalSmtStorage,
{
    fn new(
        ckb: &'a C,
        type_ids: RewardTypeIds,
        smt: S,
        info: RewardInfo,
        user: EthAddress,
        current_epoch: Epoch,
    ) -> Self {
        let token_lock = OmniEth::lock(&user);
        let xudt = Xudt::type_(&type_ids.xudt_owner.pack());

        Self {
            ckb,
            type_ids,
            smt,
            info,
            user,
            current_epoch,
            token_lock,
            xudt,
        }
    }

    async fn build_tx(&self) -> Result<TransactionView> {
        // reward smt cell
        let reward_smt_type = Reward::smt_type(&self.type_ids.reward_smt_type_id);
        let reward_smt_cell = Reward::get_cell(self.ckb, reward_smt_type.clone()).await?;
        let mut inputs = vec![CellInput::new_builder()
            .previous_output(reward_smt_cell.out_point.into())
            .build()];

        // AT cell
        let token_amount = self.add_token_to_intpus(&mut inputs).await?;

        let mut cell_deps = vec![
            OmniEth::lock_dep(),
            Secp256k1::lock_dep(),
            AlwaysSuccess::lock_dep(),
            Xudt::type_dep(),
            Reward::smt_type_dep(),
            Checkpoint::cell_dep(self.ckb, &self.type_ids.checkpoint_type_id).await?,
            Metadata::cell_dep(self.ckb, &self.type_ids.metadata_type_id).await?,
            Stake::smt_cell_dep(self.ckb, &self.type_ids.metadata_type_id).await?,
            Delegate::smt_cell_dep(self.ckb, &self.type_ids.metadata_type_id).await?,
        ];

        // 1. Build outputs data.
        // 2. Add each staker's delegate requrement cell dep.
        let outputs_data = self
            .build_data(token_amount.unwrap_or(0), &mut cell_deps)
            .await?;

        let outputs = vec![
            // reward smt cell
            CellOutput::new_builder()
                .lock(AlwaysSuccess::lock())
                .type_(Some(reward_smt_type).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[1].len())?)?,
            // AT cell
            CellOutput::new_builder()
                .lock(self.token_lock.clone())
                .type_(Some(self.xudt.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[0].len())?)?,
        ];

        let mut witnesses = vec![bytes::Bytes::default()]; // todo: reward smt cell lock
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

        let tx = Tx::new(self.ckb, tx)
            .balance(self.token_lock.clone())
            .await?;

        Ok(tx)
    }
}

impl<'a, C, S> RewardTxBuilder<'a, C, S>
where
    C: CkbRpc,
    S: RewardSmtStorage + StakeSmtStorage + DelegateSmtStorage + ProposalSmtStorage,
{
    async fn add_token_to_intpus(&self, inputs: &mut Vec<CellInput>) -> Result<Option<Amount>> {
        let (token_cells, amount) =
            Xudt::collect(self.ckb, self.token_lock.clone(), self.xudt.clone(), 1).await?;

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

    async fn build_data(
        &self,
        mut wallet_amount: Amount,
        cell_deps: &mut Vec<CellDep>,
    ) -> Result<Vec<Bytes>> {
        if self.current_epoch < INAUGURATION {
            return Err(CkbTxErr::EpochTooSmall.into());
        }

        let start_reward_epoch = self.start_reward_epoch().await?;
        let mut total_reward_amount = 0_u128;
        let user = to_eth_h160(&self.user);

        for epoch in start_reward_epoch + 1..=self.current_epoch - INAUGURATION {
            let propose_counts = ProposalSmtStorage::get_sub_leaves(&self.smt, epoch).await?;

            for (validator, propose_count) in propose_counts.into_iter() {
                let is_validator = user == validator;

                let delegate_amount = DelegateSmtStorage::get_amount(
                    &self.smt,
                    epoch + INAUGURATION,
                    validator,
                    user,
                )
                .await?;

                let in_delegate_smt = delegate_amount.is_some();

                if !is_validator && !in_delegate_smt {
                    continue;
                }

                let commission_rate = self
                    .commission_rate(&to_ckb_h160(&validator), cell_deps)
                    .await? as u128;

                let coef = if propose_count >= self.info.theoretical_propose_count * 95 / 100 {
                    100
                } else {
                    propose_count as u128 * 100 / self.info.theoretical_propose_count as u128
                };
                let total_reward = coef * self.info.base_reward
                    / (2_u64.pow((self.current_epoch / self.info.half_reward_cycle) as u32))
                        as u128
                    / 100;

                let stake_amount =
                    StakeSmtStorage::get_amount(&self.smt, epoch + INAUGURATION, validator).await?;
                if stake_amount.is_none() {
                    return Err(CkbTxErr::StakeAmountNotFound(validator).into());
                }
                let stake_amount = stake_amount.unwrap();

                let total_delegate_amount =
                    DelegateSmtStorage::get_sub_leaves(&self.smt, epoch + INAUGURATION, validator)
                        .await?
                        .values()
                        .sum::<Amount>();

                let total_amount = stake_amount + total_delegate_amount;

                if is_validator {
                    total_reward_amount += calc_validator_reward(
                        total_reward,
                        total_amount,
                        total_delegate_amount,
                        stake_amount,
                        commission_rate,
                    );
                }

                if in_delegate_smt {
                    total_reward_amount += calc_delegator_reward(
                        total_reward,
                        total_amount,
                        delegate_amount.unwrap(),
                        commission_rate,
                    );
                }
            }
        }

        wallet_amount += total_reward_amount;

        RewardSmtStorage::insert(&self.smt, self.current_epoch - INAUGURATION, user).await?;
        let reward_smt_root = RewardSmtStorage::get_root(&self.smt).await?;

        Ok(vec![
            // reward smt cell data
            RewardSmtCellData::new_builder()
                .claim_smt_root(Byte32::new_unchecked(Bytes::from(
                    reward_smt_root.as_slice().to_owned(),
                )))
                .build()
                .as_bytes(),
            // AT cell data
            wallet_amount.pack().as_bytes(),
        ])
    }

    async fn start_reward_epoch(&self) -> Result<Epoch> {
        let start_reward_epoch =
            RewardSmtStorage::get_epoch(&self.smt, to_eth_h160(&self.user)).await?;

        if start_reward_epoch.is_none() {
            return Err(CkbTxErr::RewardEpochNotFound.into());
        }

        let start_reward_epoch = start_reward_epoch.unwrap();

        let start_reward_epoch = if start_reward_epoch == 1 {
            0
        } else {
            start_reward_epoch
        };
        Ok(start_reward_epoch)
    }

    async fn commission_rate(&self, staker: &H160, cell_deps: &mut Vec<CellDep>) -> Result<u8> {
        let delegate_requirement_cell = Delegate::get_requirement_cell(
            self.ckb,
            Delegate::requirement_type(&self.type_ids.metadata_type_id, staker),
        )
        .await?;

        cell_deps.push(
            CellDep::new_builder()
                .out_point(delegate_requirement_cell.out_point.clone().into())
                .build(),
        );

        let data = delegate_requirement_cell.output_data.unwrap().into_bytes();
        let delegate_requirement = DelegateRequirement::new_unchecked(data);

        Ok(delegate_requirement.commission_rate().into())
    }
}

fn calc_validator_reward(
    total_reward: u128,
    total_amount: u128,
    total_delegate_amount: u128,
    stake_amount: u128,
    commission_rate: u128,
) -> u128 {
    let staker_reward = total_reward * stake_amount / total_amount;

    let staker_fee_reward =
        total_reward * total_delegate_amount / total_amount * (100 - commission_rate) / 100;

    staker_reward + staker_fee_reward
}

fn calc_delegator_reward(
    total_reward: u128,
    total_amount: u128,
    delegate_amount: u128,
    commission_rate: u128,
) -> u128 {
    total_reward * delegate_amount / total_amount * commission_rate / 100
}
