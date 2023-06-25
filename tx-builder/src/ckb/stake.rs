use anyhow::Result;
use async_trait::async_trait;
use axon_types::{
    delegate::DelegateCellData, stake::StakeAtCellData as AStakeAtCellData,
    withdraw::WithdrawAtCellData,
};
use ckb_types::{
    bytes::Bytes,
    core::{Capacity, TransactionBuilder, TransactionView},
    packed::{CellInput, CellOutput, Script},
    prelude::{Entity, Pack},
};
use molecule::prelude::Builder;

use common::traits::ckb_rpc_client::CkbRpc;
use common::traits::tx_builder::IStakeTxBuilder;
use common::types::ckb_rpc_client::Cell;
use common::types::tx_builder::*;
use common::utils::convert::*;

use crate::ckb::define::constants::*;
use crate::ckb::define::error::{CkbTxErr, CkbTxResult};
use crate::ckb::define::types::StakeAtCellData;
use crate::ckb::utils::{
    calc_amount::*,
    cell_collector::{collect_xudt, get_stake_cell, get_withdraw_cell},
    cell_data::*,
    cell_dep::*,
    omni::*,
    script::*,
    tx::balance_tx,
};

pub struct StakeTxBuilder<C: CkbRpc> {
    ckb:              CkbNetwork<C>,
    type_ids:         StakeTypeIds,
    current_epoch:    Epoch,
    stake:            StakeItem,
    first_stake_info: Option<FirstStakeInfo>,
    stake_lock:       Script,
    token_lock:       Script,
    withdraw_lock:    Script,
    xudt:             Script,
}

#[async_trait]
impl<C: CkbRpc> IStakeTxBuilder<C> for StakeTxBuilder<C> {
    fn new(
        ckb: CkbNetwork<C>,
        type_ids: StakeTypeIds,
        staker: EthAddress,
        current_epoch: Epoch,
        stake_item: StakeItem,
        first_stake_info: Option<FirstStakeInfo>,
    ) -> Self {
        let stake_lock = stake_lock(&ckb.network_type, &type_ids.metadata_type_id, &staker);
        let withdraw_lock = withdraw_lock(&ckb.network_type, &type_ids.metadata_type_id, &staker);
        let token_lock = omni_eth_lock(&ckb.network_type, &staker);
        let xudt = xudt_type(&ckb.network_type, &type_ids.xudt_owner.pack());

        Self {
            ckb,
            type_ids,
            current_epoch,
            stake: stake_item,
            first_stake_info,
            stake_lock,
            token_lock,
            withdraw_lock,
            xudt,
        }
    }

    async fn build_tx(&self) -> Result<TransactionView> {
        if self.stake.inauguration_epoch > self.current_epoch + INAUGURATION {
            return Err(CkbTxErr::InaugurationEpoch {
                expected: self.current_epoch,
                found:    self.stake.inauguration_epoch,
            }
            .into());
        }

        let stake_cell =
            get_stake_cell(&self.ckb.client, self.stake_lock.clone(), self.xudt.clone()).await?;
        if stake_cell.is_none() {
            self.build_first_stake_tx().await
        } else {
            self.build_update_stake_tx(stake_cell.unwrap()).await
        }
    }
}

impl<C: CkbRpc> StakeTxBuilder<C> {
    async fn build_first_stake_tx(&self) -> Result<TransactionView> {
        let mut inputs = vec![];

        // AT cells
        let token_amount = self.add_token_to_inputs(&mut inputs).await?;

        let mut outputs_data = self.first_stake_data(token_amount)?;

        let mut outputs = vec![
            // AT cell
            CellOutput::new_builder()
                .lock(self.token_lock.clone())
                .type_(Some(self.xudt.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[0].len())?)?,
            // stake AT cell
            CellOutput::new_builder()
                .lock(self.stake_lock.clone())
                .type_(Some(self.xudt.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[1].len())?)?,
            // delegate requirement cell
            CellOutput::new_builder()
                .lock(self.token_lock.clone())
                // .type_(Some(fake_type.clone()).pack()) // todo
                .build_exact_capacity(Capacity::bytes(outputs_data[2].len())?)?,
        ];

        self.add_withdraw_to_outputs(&mut outputs, &mut outputs_data)
            .await?;

        let cell_deps = vec![
            omni_lock_dep(&self.ckb.network_type),
            secp256k1_lock_dep(&self.ckb.network_type),
            xudt_type_dep(&self.ckb.network_type),
        ];

        let witnesses = vec![
            omni_eth_witness_placeholder().as_bytes(), // AT cell lock
            omni_eth_witness_placeholder().as_bytes(), // capacity provider lock
        ];

        let tx = TransactionBuilder::default()
            .inputs(inputs)
            .outputs(outputs)
            .outputs_data(outputs_data.pack())
            .cell_deps(cell_deps)
            .witnesses(witnesses.pack())
            .build();

        let tx = balance_tx(&self.ckb.client, self.token_lock.clone(), tx).await?;

        Ok(tx)
    }

    async fn build_update_stake_tx(&self, stake_cell: Cell) -> Result<TransactionView> {
        // stake AT cell
        let mut inputs = vec![CellInput::new_builder()
            .previous_output(stake_cell.out_point.into())
            .build()];

        // AT cells
        let token_amount = self.add_token_to_inputs(&mut inputs).await?;

        let stake_data = stake_cell.output_data.unwrap().into_bytes();
        let outputs_data = self.update_stake_data(token_amount, stake_data)?;

        let outputs = vec![
            // stake AT cell
            CellOutput::new_builder()
                .lock(self.stake_lock.clone())
                .type_(Some(self.xudt.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[0].len())?)?,
            // AT cell
            CellOutput::new_builder()
                .lock(self.token_lock.clone())
                .type_(Some(self.xudt.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[1].len())?)?,
        ];

        let cell_deps = vec![
            omni_lock_dep(&self.ckb.network_type),
            secp256k1_lock_dep(&self.ckb.network_type),
            xudt_type_dep(&self.ckb.network_type),
            stake_lock_dep(&self.ckb.network_type),
            checkpoint_cell_dep(
                &self.ckb.client,
                &self.ckb.network_type,
                &self.type_ids.checkpoint_type_id,
            )
            .await?,
            metadata_cell_dep(
                &self.ckb.client,
                &self.ckb.network_type,
                &self.type_ids.metadata_type_id,
            )
            .await?,
        ];

        let witnesses = vec![
            stake_witness_placeholder(0u8).as_bytes(), // stake AT cell lock, todo
            omni_eth_witness_placeholder().as_bytes(), // AT cell lock
            omni_eth_witness_placeholder().as_bytes(), // capacity provider lock
        ];

        let tx = TransactionBuilder::default()
            .inputs(inputs)
            .outputs(outputs)
            .outputs_data(outputs_data.pack())
            .cell_deps(cell_deps)
            .witnesses(witnesses.pack())
            .build();

        let tx = balance_tx(&self.ckb.client, self.token_lock.clone(), tx).await?;

        Ok(tx)
    }

    async fn add_token_to_inputs(&self, inputs: &mut Vec<CellInput>) -> Result<Amount> {
        let (token_cells, amount) = collect_xudt(
            &self.ckb.client,
            self.token_lock.clone(),
            self.xudt.clone(),
            self.stake.amount,
        )
        .await?;

        if token_cells.is_empty() {
            return Err(CkbTxErr::CellNotFound("AT".to_owned()).into());
        }

        // AT cells
        for token_cell in token_cells.into_iter() {
            inputs.push(
                CellInput::new_builder()
                    .previous_output(token_cell.out_point.into())
                    .build(),
            );
        }

        Ok(amount)
    }

    async fn add_withdraw_to_outputs(
        &self,
        outputs: &mut Vec<CellOutput>,
        outputs_data: &mut Vec<Bytes>,
    ) -> Result<()> {
        let withdraw_cell = get_withdraw_cell(
            &self.ckb.client,
            self.withdraw_lock.clone(),
            self.xudt.clone(),
        )
        .await?;

        if withdraw_cell.is_none() {
            outputs_data.push(token_cell_data(0, WithdrawAtCellData::default().as_bytes()));
            outputs.push(
                CellOutput::new_builder()
                    .lock(self.withdraw_lock.clone())
                    .type_(Some(self.xudt.clone()).pack())
                    .build_exact_capacity(Capacity::bytes(outputs_data.last().unwrap().len())?)?,
            );
        }

        Ok(())
    }

    fn first_stake_data(&self, mut wallet_amount: Amount) -> CkbTxResult<Vec<Bytes>> {
        if !self.stake.is_increase {
            return Err(CkbTxErr::Increase(self.stake.is_increase));
        }

        if wallet_amount < self.stake.amount {
            return Err(CkbTxErr::ExceedWalletAmount {
                wallet_amount,
                amount: self.stake.amount,
            });
        }
        wallet_amount -= self.stake.amount;

        let first_stake = self.first_stake_info.as_ref().ok_or(CkbTxErr::FirstStake)?;

        Ok(vec![
            // AT cell data
            wallet_amount.pack().as_bytes(),
            // stake AT cell data
            token_cell_data(
                self.stake.amount,
                AStakeAtCellData::from(StakeAtCellData {
                    l1_pub_key:  first_stake.l1_pub_key.clone(),
                    bls_pub_key: first_stake.bls_pub_key.clone(),
                    stake_info:  self.stake.clone(),
                })
                .as_bytes(),
            ),
            // delegate requirement cell data
            DelegateCellData::new_builder()
                .delegate_requirement(first_stake.delegate.clone().into())
                .build()
                .as_bytes(),
        ])
    }

    fn update_stake_data(
        &self,
        wallet_amount: Amount,
        stake_data: Bytes,
    ) -> CkbTxResult<Vec<Bytes>> {
        let total_stake_amount = new_u128(&stake_data[..TOKEN_BYTES]);

        let mut stake_data = stake_data;
        let stake_data = AStakeAtCellData::new_unchecked(stake_data.split_off(TOKEN_BYTES));
        let last_info =
            ElectAmountCalculator::last_stake_info(&stake_data.delta(), self.current_epoch);

        let actual_info = ElectAmountCalculator::new(
            wallet_amount,
            total_stake_amount,
            last_info,
            ElectItem::Stake(&self.stake),
        )
        .calc_actual_amount()?;

        Ok(vec![
            // stake AT cell data
            token_cell_data(
                actual_info.total_elect_amount,
                AStakeAtCellData::from(StakeAtCellData {
                    l1_pub_key:  stake_data.l1_pub_key(),
                    bls_pub_key: stake_data.bls_pub_key(),
                    stake_info:  StakeItem {
                        is_increase:        actual_info.is_increase,
                        amount:             actual_info.amount,
                        inauguration_epoch: self.stake.inauguration_epoch,
                    },
                })
                .as_bytes(),
            ),
            // AT cell data
            actual_info.wallet_amount.pack().as_bytes(),
        ])
    }
}
