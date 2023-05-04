use anyhow::Result;
use async_trait::async_trait;
use axon_types::stake::StakeAtCellData;
use ckb_types::{
    bytes::Bytes,
    core::{Capacity, TransactionBuilder, TransactionView},
    packed::{CellOutput, Script},
    prelude::{Entity, Pack},
};

use common::traits::tx_builder::IStakeTxBuilder;
use common::types::tx_builder::*;
use common::utils::convert::*;

use crate::ckb::define::config::*;
use crate::ckb::define::error::{CkbTxErr, CkbTxResult};
use crate::ckb::utils::{calc_amount::*, cell_data::*};

pub struct StakeTxBuilder {
    _staker:       Address,
    current_epoch: Epoch,
    stake:         StakeItem,
    delegate:      Option<StakeDelegate>,
}

#[async_trait]
impl IStakeTxBuilder for StakeTxBuilder {
    fn new(
        _staker: Address,
        current_epoch: Epoch,
        stake_item: StakeItem,
        delegate: Option<StakeDelegate>,
    ) -> Self {
        Self {
            _staker,
            current_epoch,
            stake: stake_item,
            delegate,
        }
    }

    async fn build_tx(&self) -> Result<TransactionView> {
        if self.stake.inauguration_epoch > self.current_epoch + 2 {
            return Err(CkbTxErr::InaugurationEpoch {
                expected: self.current_epoch,
                found:    self.stake.inauguration_epoch,
            }
            .into());
        }

        if self.is_first_stake().await? {
            self.build_first_stake_tx().await
        } else {
            self.build_update_stake_tx().await
        }
    }
}

impl StakeTxBuilder {
    async fn is_first_stake(&self) -> Result<bool> {
        // todo: search user's stake AT cell
        Ok(true)
    }

    async fn build_first_stake_tx(&self) -> Result<TransactionView> {
        // todo: collect AT cells
        let inputs = vec![];

        let wallet_data = vec![Bytes::default()]; // todo: from inputs
        let outputs_data = self.first_stake_data(&wallet_data)?;

        // todo: fill lock, type
        let fake_lock = Script::default();
        let fake_type = Script::default();
        let outputs = vec![
            // AT cell
            CellOutput::new_builder()
                .lock(fake_lock.clone())
                .type_(Some(fake_type.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[0].len())?)?,
            // stake AT cell
            CellOutput::new_builder()
                .lock(fake_lock.clone())
                .type_(Some(fake_type.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[1].len())?)?,
            // delegate cell
            CellOutput::new_builder()
                .lock(fake_lock.clone())
                .type_(Some(fake_type.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[2].len())?)?,
            // withdraw AT cell
            CellOutput::new_builder()
                .lock(fake_lock)
                .type_(Some(fake_type).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[3].len())?)?,
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

        Ok(tx)
    }

    async fn build_update_stake_tx(&self) -> Result<TransactionView> {
        // todo: collect AT cells
        // todo: get stake AT cell
        let inputs = vec![];

        let wallet_data = vec![Bytes::default()]; // todo
        let stake_data = Bytes::default(); // todo
        let outputs_data = self.update_stake_data(&wallet_data, stake_data)?;

        // todo: fill lock, type
        let fake_lock = Script::default();
        let fake_type = Script::default();
        let outputs = vec![
            // AT cell
            CellOutput::new_builder()
                .lock(fake_lock.clone())
                .type_(Some(fake_type.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[0].len())?)?,
            // stake AT cell
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

        Ok(tx)
    }

    fn first_stake_data(&self, wallet_data: &[Bytes]) -> CkbTxResult<Vec<Bytes>> {
        if !self.stake.is_increase {
            return Err(CkbTxErr::Increase(self.stake.is_increase));
        }

        let mut wallet_amount = ElectAmountCaculator::calc_wallet_amount(wallet_data);
        if wallet_amount < self.stake.amount {
            return Err(CkbTxErr::ExceedWalletAmount {
                wallet_amount,
                amount: self.stake.amount,
            });
        }
        wallet_amount -= self.stake.amount;

        let delegate = self.delegate.as_ref().ok_or(CkbTxErr::Delegate)?;

        Ok(vec![
            // AT cell data
            to_uint128(wallet_amount).as_bytes(),
            // stake AT cell data
            token_cell_data(
                self.stake.amount,
                stake_token_cell_data(
                    self.stake.is_increase,
                    self.stake.amount,
                    self.stake.inauguration_epoch,
                )
                .as_bytes(),
            ),
            // delegate cell data
            delegate_cell_data(
                delegate.threshold,
                delegate.maximum_delegators,
                delegate.dividend_ratio,
            )
            .as_bytes(),
            // withdraw AT cell data
            token_cell_data(0, withdraw_token_cell_data(None).as_bytes()),
        ])
    }

    fn update_stake_data(
        &self,
        wallet_data: &[Bytes],
        stake_data: Bytes,
    ) -> CkbTxResult<Vec<Bytes>> {
        let wallet_amount = ElectAmountCaculator::calc_wallet_amount(wallet_data);
        let total_stake_amount = new_u128(&stake_data[..TOKEN_BYTES]);

        let mut stake_data = stake_data;
        let stake_data = StakeAtCellData::new_unchecked(stake_data.split_off(TOKEN_BYTES));
        let last_info =
            ElectAmountCaculator::last_stake_info(&stake_data.stake_info(), self.current_epoch);

        let actual_info = ElectAmountCaculator::new(
            wallet_amount,
            total_stake_amount,
            last_info,
            ElectItem::Stake(&self.stake),
        )
        .calc_actual_amount()?;

        Ok(vec![
            // AT cell data
            actual_info.wallet_amount.pack().as_bytes(),
            // stake AT cell data
            token_cell_data(
                actual_info.total_elect_amount,
                stake_token_cell_data(
                    actual_info.is_increase,
                    actual_info.amount,
                    self.stake.inauguration_epoch,
                )
                .as_bytes(),
            ),
        ])
    }
}
