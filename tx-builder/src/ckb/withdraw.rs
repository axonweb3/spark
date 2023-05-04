use anyhow::Result;
use async_trait::async_trait;
use axon_types::withdraw::*;
use ckb_types::{
    bytes::Bytes,
    core::{Capacity, TransactionBuilder, TransactionView},
    packed::{CellOutput, Script},
    prelude::{Builder, Entity, Pack},
};

use common::traits::tx_builder::IWithdrawTxBuilder;
use common::types::tx_builder::{Address, Epoch};
use common::utils::convert::*;

use crate::ckb::define::config::{INAUGURATION, TOKEN_BYTES};
use crate::ckb::define::error::CkbTxResult;
use crate::ckb::utils::{calc_amount::*, cell_data::*};

pub struct WithdrawTxBuilder {
    _user:         Address,
    current_epoch: Epoch,
}

#[async_trait]
impl IWithdrawTxBuilder for WithdrawTxBuilder {
    fn new(_user: Address, current_epoch: Epoch) -> Self {
        Self {
            _user,
            current_epoch,
        }
    }

    async fn build_tx(&self) -> Result<TransactionView> {
        // todo: collect AT cells
        // todo: get withdraw AT cell
        let inputs = vec![];

        let wallet_data = vec![Bytes::default()]; // todo
        let withdraw_data = Bytes::default(); // todo
        let outputs_data = self.build_data(&wallet_data, withdraw_data).await?;

        // todo: fill lock, type
        let fake_lock = Script::default();
        let fake_type = Script::default();
        let outputs = vec![
            // AT cell
            CellOutput::new_builder()
                .lock(fake_lock.clone())
                .type_(Some(fake_type.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[0].len())?)?,
            // withdraw AT cell
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
}

impl WithdrawTxBuilder {
    async fn build_data(
        &self,
        wallet_data: &[Bytes],
        withdraw_data: Bytes,
    ) -> CkbTxResult<Vec<Bytes>> {
        let mut wallet_amount = ElectAmountCaculator::calc_wallet_amount(wallet_data);
        let mut total_withdraw_amount = new_u128(&withdraw_data[..TOKEN_BYTES]);

        let mut withdraw_data = withdraw_data;
        let cell_withdraws =
            WithdrawAtCellData::new_unchecked(withdraw_data.split_off(TOKEN_BYTES));

        let mut output_withdraw_infos = WithdrawInfos::new_builder();
        let mut unlock_amount = 0;

        for withdraw_info in cell_withdraws.withdraw_infos() {
            let epoch = to_u64(withdraw_info.epoch());
            if epoch <= self.current_epoch - INAUGURATION {
                unlock_amount += to_u128(withdraw_info.amount());
            } else {
                output_withdraw_infos = output_withdraw_infos.push(withdraw_info);
            }
        }

        wallet_amount += unlock_amount;
        total_withdraw_amount -= unlock_amount;

        Ok(vec![
            // AT cell data
            wallet_amount.pack().as_bytes(),
            // withdraw AT cell data
            token_cell_data(
                total_withdraw_amount,
                withdraw_token_cell_data(Some(output_withdraw_infos.build())).as_bytes(),
            ),
        ])
    }
}
