use anyhow::Result;
use async_trait::async_trait;
use ckb_types::{
    bytes::Bytes,
    core::{Capacity, TransactionBuilder, TransactionView},
    packed::{CellInput, CellOutput, Script},
    prelude::{Builder, Entity, Pack},
};

use common::traits::ckb_rpc_client::CkbRpc;
use common::traits::tx_builder::IWithdrawTxBuilder;
use common::types::axon_types::withdraw::*;
use common::types::ckb_rpc_client::Cell;
use common::types::tx_builder::{Amount, Epoch, EthAddress, StakeTypeIds};
use common::utils::convert::*;

use crate::ckb::define::constants::TOKEN_BYTES;
use crate::ckb::define::error::CkbTxResult;
use crate::ckb::helper::{
    token_cell_data, Checkpoint, Metadata, OmniEth, Secp256k1, Tx, Withdraw, Xudt,
};

use super::define::error::CkbTxErr;

pub struct WithdrawTxBuilder<'a, C: CkbRpc> {
    ckb:           &'a C,
    type_ids:      StakeTypeIds,
    current_epoch: Epoch,
    withdraw_lock: Script,
    token_lock:    Script,
    xudt:          Script,
}

#[async_trait]
impl<'a, C: CkbRpc> IWithdrawTxBuilder<'a, C> for WithdrawTxBuilder<'a, C> {
    fn new(ckb: &'a C, type_ids: StakeTypeIds, user: EthAddress, current_epoch: Epoch) -> Self {
        let withdraw_lock = Withdraw::lock(&type_ids.metadata_type_id, &user);
        let token_lock = OmniEth::lock(&user);
        let xudt = Xudt::type_(&type_ids.xudt_owner.pack());

        Self {
            ckb,
            type_ids,
            current_epoch,
            withdraw_lock,
            token_lock,
            xudt,
        }
    }

    async fn build_tx(&self) -> Result<TransactionView> {
        let withdraw_cell = self.get_withdraw_cell().await?;

        // withdraw AT cell
        let mut inputs = vec![CellInput::new_builder()
            .previous_output(withdraw_cell.out_point.into())
            .build()];

        // AT cell
        let token_amount = self.add_token_to_inputs(&mut inputs).await?;

        let withdraw_data = withdraw_cell.output_data.unwrap().into_bytes();
        let outputs_data = self.build_data(token_amount, withdraw_data).await?;

        let outputs = vec![
            // withdraw AT cell
            CellOutput::new_builder()
                .lock(self.withdraw_lock.clone())
                .type_(Some(self.xudt.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[0].len())?)?,
            // AT cell
            CellOutput::new_builder()
                .lock(self.token_lock.clone())
                .type_(Some(self.xudt.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[1].len())?)?,
        ];

        let cell_deps = vec![
            OmniEth::lock_dep(),
            Secp256k1::lock_dep(),
            Xudt::type_dep(),
            Withdraw::lock_dep(),
            Checkpoint::cell_dep(self.ckb, &self.type_ids.checkpoint_type_id).await?,
            Metadata::cell_dep(self.ckb, &self.type_ids.metadata_type_id).await?,
        ];

        let witnesses = vec![
            OmniEth::witness_placeholder().as_bytes(), // withdraw AT cell lock
            OmniEth::witness_placeholder().as_bytes(), // AT cell lock
            OmniEth::witness_placeholder().as_bytes(), // capacity provider lock
        ];

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

impl<'a, C: CkbRpc> WithdrawTxBuilder<'a, C> {
    async fn get_withdraw_cell(&self) -> Result<Cell> {
        let withdraw_cell =
            Withdraw::get_cell(self.ckb, self.withdraw_lock.clone(), self.xudt.clone()).await?;

        if withdraw_cell.is_none() {
            return Err(CkbTxErr::CellNotFound("Withdraw".to_owned()).into());
        }

        Ok(withdraw_cell.unwrap())
    }

    async fn add_token_to_inputs(&self, inputs: &mut Vec<CellInput>) -> Result<Amount> {
        let (token_cells, amount) =
            Xudt::collect(self.ckb, self.token_lock.clone(), self.xudt.clone(), 1).await?;

        if token_cells.is_empty() {
            return Err(CkbTxErr::CellNotFound("AT".to_owned()).into());
        }

        // AT cell
        inputs.push(
            CellInput::new_builder()
                .previous_output(token_cells[0].out_point.clone().into())
                .build(),
        );

        Ok(amount)
    }

    async fn build_data(
        &self,
        mut wallet_amount: Amount,
        mut withdraw_data: Bytes,
    ) -> CkbTxResult<Vec<Bytes>> {
        let mut total_withdraw_amount = new_u128(&withdraw_data[..TOKEN_BYTES]);

        let withdraw_data = WithdrawAtCellData::new_unchecked(withdraw_data.split_off(TOKEN_BYTES));

        let mut output_withdraw_infos = WithdrawInfos::new_builder();
        let mut unlock_amount = 0;

        for withdraw_info in withdraw_data.lock().withdraw_infos() {
            let epoch = to_u64(&withdraw_info.unlock_epoch());
            if epoch <= self.current_epoch {
                unlock_amount += to_u128(&withdraw_info.amount());
            } else {
                output_withdraw_infos = output_withdraw_infos.push(withdraw_info);
            }
        }

        wallet_amount += unlock_amount;
        total_withdraw_amount -= unlock_amount;

        let inner_withdraw_data = withdraw_data.lock();

        Ok(vec![
            // withdraw AT cell data
            token_cell_data(
                total_withdraw_amount,
                withdraw_data
                    .as_builder()
                    .lock(
                        inner_withdraw_data
                            .as_builder()
                            .withdraw_infos(output_withdraw_infos.build())
                            .build(),
                    )
                    .build()
                    .as_bytes(),
            ),
            // AT cell data
            wallet_amount.pack().as_bytes(),
        ])
    }
}
