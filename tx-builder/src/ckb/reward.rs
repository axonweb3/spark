use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use axon_types::basic::*;
use ckb_types::{
    bytes::Bytes,
    core::{Capacity, TransactionBuilder, TransactionView},
    packed::{CellOutput, Script},
    prelude::{Entity, Pack},
    H160,
};

use common::traits::tx_builder::IRewardTxBuilder;
use common::types::tx_builder::{Address, Amount, Epoch};
use common::utils::convert::new_u128;

use crate::ckb::define::config::{INAUGURATION, TOKEN_BYTES};
use crate::ckb::define::error::CkbTxResult;

pub struct RewardTxBuilder {
    user:                      Address,
    current_epoch:             Epoch,
    base_reward:               Amount,
    half_reward_cycle:         Epoch,
    theoretical_propose_count: u64,
}

#[async_trait]
impl IRewardTxBuilder for RewardTxBuilder {
    fn new(
        user: Address,
        current_epoch: Epoch,
        base_reward: u128,
        half_reward_cycle: Epoch,
        theoretical_propose_count: u64,
    ) -> Self {
        Self {
            user,
            current_epoch,
            base_reward,
            half_reward_cycle,
            theoretical_propose_count,
        }
    }

    // todo: split tx
    async fn build_txs(&self) -> Result<Vec<TransactionView>> {
        // todo: get AT cell
        // todo: get reward SMT cell
        let inputs = vec![];

        let wallet_data = Bytes::default(); // todo
        let outputs_data = self.build_data(&wallet_data)?;

        // todo: fill lock, type
        let fake_lock = Script::default();
        let fake_type = Script::default();
        let outputs = vec![
            // AT cell
            CellOutput::new_builder()
                .lock(fake_lock.clone())
                .type_(Some(fake_type.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[0].len())?)?,
            // reward SMT cell
            CellOutput::new_builder()
                .lock(fake_lock)
                .type_(Some(fake_type).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[1].len())?)?,
        ];

        // todo
        let cell_deps = vec![];

        // todo: balance tx, fill placeholder witnesses
        let tx = TransactionBuilder::default()
            .inputs(inputs)
            .outputs(outputs)
            .outputs_data(outputs_data.pack())
            .cell_deps(cell_deps)
            .build();

        Ok(vec![tx])
    }
}

impl RewardTxBuilder {
    fn build_data(&self, wallet_data: &Bytes) -> CkbTxResult<Vec<Bytes>> {
        let start_reward_epoch: Epoch = 3; // todo: get from reward smt
        let start_reward_epoch = if start_reward_epoch == 1 {
            0
        } else {
            start_reward_epoch
        };

        let mut total_reward_amount = 0_u128;

        for _epoch in start_reward_epoch + 1..=self.current_epoch - INAUGURATION {
            let propose_counts: HashMap<H160, u64> = HashMap::new(); // todo: get from proposal smt

            for (proposer, propose_count) in propose_counts.into_iter() {
                let is_validator = self.user == proposer;

                // todo: get it from smt
                let in_delegate_smt = true;

                if !is_validator && !in_delegate_smt {
                    continue;
                }

                // todo: get dividend ratio from staker's delegate cell
                let dividend_ratio = 80;

                let coef = if propose_count >= self.theoretical_propose_count * 95 / 100 {
                    100
                } else {
                    propose_count as u128 * 100 / self.theoretical_propose_count as u128
                };
                let total_reward = coef * self.base_reward
                    / (2_u64.pow((self.current_epoch / self.half_reward_cycle) as u32)) as u128
                    / 100;

                let stake_amount = 200_u128; // todo: get from stake smt
                let total_amount = 100_u128; // todo: get from delegate smt
                let total_stake_amount = stake_amount + total_amount;

                if is_validator {
                    total_reward_amount += calc_validator_reward(
                        total_reward,
                        total_stake_amount,
                        total_amount,
                        stake_amount,
                        dividend_ratio,
                    );
                }

                if in_delegate_smt {
                    let delegator_amount = 50_u128; // todo: get from delegate smt
                    total_reward_amount += calc_delegator_reward(
                        total_reward,
                        total_stake_amount,
                        delegator_amount,
                        dividend_ratio,
                    );
                }
            }
        }

        let mut wallet_amount = new_u128(&wallet_data[..TOKEN_BYTES]);
        wallet_amount += total_reward_amount;

        // todo: generate new reward root
        let new_root = Byte32::default();

        Ok(vec![
            // AT cell data
            wallet_amount.pack().as_bytes(),
            // reward cell data
            new_root.as_bytes(),
        ])
    }
}

fn calc_validator_reward(
    total_reward: u128,
    total_stake_amount: u128,
    total_amount: u128,
    stake_amount: u128,
    dividend_ratio: u128,
) -> u128 {
    let staker_reward = total_reward * stake_amount / total_stake_amount;

    let staker_fee_reward =
        total_reward * total_amount / total_stake_amount * (100 - dividend_ratio) / 100;

    staker_reward + staker_fee_reward
}

fn calc_delegator_reward(
    total_reward: u128,
    total_stake_amount: u128,
    delegator_amount: u128,
    dividend_ratio: u128,
) -> u128 {
    total_reward * delegator_amount / total_stake_amount * dividend_ratio / 100
}
