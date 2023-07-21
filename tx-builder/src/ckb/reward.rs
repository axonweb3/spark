use std::cmp::min;

use anyhow::Result;
use async_trait::async_trait;
use ckb_types::{
    bytes::Bytes,
    core::{Capacity, TransactionBuilder, TransactionView},
    packed::{CellDep, CellInput, CellOutput, Script, WitnessArgs},
    prelude::{Entity, Pack},
    H160,
};
use molecule::prelude::Builder;

use common::{
    traits::ckb_rpc_client::CkbRpc,
    traits::smt::{DelegateSmtStorage, ProposalSmtStorage, RewardSmtStorage, StakeSmtStorage},
    traits::tx_builder::IRewardTxBuilder,
    types::axon_types::{
        delegate::DelegateRequirement,
        reward::{RewardSmtCellData as ARewardSmtCellData, RewardWitness as ARewardWitness},
    },
    types::tx_builder::{Amount, Epoch, EthAddress, RewardInfo, RewardTypeIds},
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

    async fn build_tx(self) -> Result<TransactionView> {
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
        // 3. Build witness for reward smt cell.
        let (mut outputs_data, reward_witness) = self
            .build_data_and_witness(token_amount.unwrap_or(0), &mut cell_deps)
            .await?;
        outputs_data.push(selection_cell.output_data.unwrap().into_bytes());

        let outputs = vec![
            // reward smt cell
            CellOutput::new_builder()
                .lock(AlwaysSuccess::lock())
                .type_(Some(Reward::smt_type(&self.type_ids.reward_smt_type_id)).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[0].len())?)?,
            // AT cell
            CellOutput::new_builder()
                .lock(self.token_lock.clone())
                .type_(Some(self.xudt.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[1].len())?)?,
            // selection cell
            CellOutput::new_builder()
                .lock(AlwaysSuccess::lock())
                .type_(Some(Selection::type_(&self.type_ids.selection_type_id)).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[2].len())?)?,
        ];

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
        tx.balance(self.token_lock.clone()).await?;

        Ok(tx.inner())
    }
}

impl<'a, C, S> RewardTxBuilder<'a, C, S>
where
    C: CkbRpc,
    S: RewardSmtStorage + StakeSmtStorage + DelegateSmtStorage + ProposalSmtStorage,
{
    async fn add_token_to_inputs(&self, inputs: &mut Vec<CellInput>) -> Result<Option<Amount>> {
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

    async fn build_data_and_witness(
        &self,
        mut wallet_amount: Amount,
        cell_deps: &mut Vec<CellDep>,
    ) -> Result<(Vec<Bytes>, RewardWitness)> {
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
            start_reward_epoch + self.info.epoch_count - 1,
        );

        let mut total_reward_amount = 0_u128;
        let user = to_eth_h160(&self.user);

        for epoch in start_reward_epoch..=end_reward_epoch {
            let propose_counts = ProposalSmtStorage::get_sub_leaves(&self.smt, epoch).await?;

            let mut epoch_reward_witness = EpochRewardStakeInfo::default();
            let mut has_reward = true;
            let mut validators = vec![];

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
                    has_reward = false;
                    continue;
                }

                validators.push(validator);

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

                let all_delegates =
                    DelegateSmtStorage::get_sub_leaves(&self.smt, epoch + INAUGURATION, validator)
                        .await?;
                let total_delegate_amount = all_delegates.values().sum::<Amount>();

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
                        .await?,
                    });
            }

            if has_reward {
                epoch_reward_witness.count_proof =
                    ProposalSmtStorage::generate_sub_proof(&self.smt, epoch, validators.clone())
                        .await?;
                epoch_reward_witness.count_root =
                    ProposalSmtStorage::get_sub_root(&self.smt, epoch)
                        .await?
                        .unwrap();
                epoch_reward_witness.count_epoch_proof =
                    ProposalSmtStorage::generate_top_proof(&self.smt, vec![epoch]).await?;
                epoch_reward_witness.amount_proof =
                    StakeSmtStorage::generate_sub_proof(&self.smt, epoch, validators).await?;
                epoch_reward_witness.amount_root = StakeSmtStorage::get_top_root(&self.smt).await?;
                epoch_reward_witness.amount_epoch_proof =
                    StakeSmtStorage::generate_top_proof(&self.smt, vec![epoch]).await?;

                witness.reward_infos.push(epoch_reward_witness);
            }
        }

        wallet_amount += total_reward_amount;

        RewardSmtStorage::insert(&self.smt, end_reward_epoch, user).await?;
        witness.new_not_claim_info = NotClaimInfo {
            epoch: end_reward_epoch,
            proof: RewardSmtStorage::generate_proof(&self.smt, vec![to_eth_h160(&self.user)])
                .await?,
        };

        let reward_smt_root = RewardSmtStorage::get_root(&self.smt).await?;

        Ok((
            vec![
                // reward smt cell data
                ARewardSmtCellData::from(RewardSmtCellData {
                    claim_smt_root:     reward_smt_root,
                    metadata_type_hash: Reward::smt_type(&self.type_ids.metadata_type_id)
                        .calc_script_hash(),
                })
                .as_bytes(),
                // AT cell data
                wallet_amount.pack().as_bytes(),
            ],
            witness,
        ))
    }

    async fn get_start_epoch(&self, witness: &mut RewardWitness) -> Result<Epoch> {
        let start_reward_epoch =
            RewardSmtStorage::get_epoch(&self.smt, to_eth_h160(&self.user)).await?;

        if start_reward_epoch.is_none() {
            return Ok(START_EPOCH);
        }

        witness.old_not_claim_info = NotClaimInfo {
            epoch: start_reward_epoch.unwrap(),
            proof: RewardSmtStorage::generate_proof(&self.smt, vec![to_eth_h160(&self.user)])
                .await?,
        };

        Ok(start_reward_epoch.unwrap() + 1)
    }

    async fn commission_rate(&self, staker: &H160, cell_deps: &mut Vec<CellDep>) -> Result<u8> {
        let requirement_type_id = Stake::get_delegate_requirement_type_id(
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
