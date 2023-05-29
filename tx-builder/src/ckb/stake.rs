use anyhow::Result;
use async_trait::async_trait;
use axon_types::stake::StakeAtCellData;
use ckb_types::{
    bytes::Bytes,
    core::{Capacity, TransactionBuilder, TransactionView},
    packed::{CellInput, CellOutput, Script},
    prelude::{Entity, Pack},
};
use molecule::prelude::Builder;

use common::traits::ckb_rpc_client::CkbRpc;
use common::traits::tx_builder::IStakeTxBuilder;
use common::types::ckb_rpc_client::{Cell, ScriptType, SearchKey, SearchKeyFilter};
use common::types::tx_builder::*;
use common::utils::convert::*;

use crate::ckb::define::config::*;
use crate::ckb::define::error::{CkbTxErr, CkbTxResult};
use crate::ckb::utils::{
    calc_amount::*,
    cell_collector::{collect_cells, collect_xudt},
    cell_data::*,
    cell_dep::*,
    omni::*,
    script::*,
    tx::balance_tx,
};

pub struct StakeTxBuilder<C: CkbRpc> {
    ckb:           CkbNetwork<C>,
    type_ids:      StakeTypeIds,
    current_epoch: Epoch,
    stake:         StakeItem,
    delegate:      Option<DelegateRequirement>,
    stake_lock:    Script,
    token_lock:    Script,
    xudt:          Script,
}

#[async_trait]
impl<C: CkbRpc> IStakeTxBuilder<C> for StakeTxBuilder<C> {
    fn new(
        ckb: CkbNetwork<C>,
        type_ids: StakeTypeIds,
        staker: EthAddress,
        current_epoch: Epoch,
        stake_item: StakeItem,
        delegate: Option<DelegateRequirement>,
    ) -> Self {
        let stake_lock = stake_lock(&ckb.network_type, &type_ids.metadata_type_id, &staker);
        let token_lock = omni_eth_lock(&ckb.network_type, &staker);
        let xudt = xudt_type(&ckb.network_type, &type_ids.xudt_owner.pack());

        Self {
            ckb,
            type_ids,
            current_epoch,
            stake: stake_item,
            delegate,
            stake_lock,
            token_lock,
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

        let stake_cell = self.get_stake_cell().await?;

        if stake_cell.is_empty() {
            self.build_first_stake_tx().await
        } else {
            self.build_update_stake_tx(stake_cell[0].clone()).await
        }
    }
}

impl<C: CkbRpc> StakeTxBuilder<C> {
    async fn get_stake_cell(&self) -> Result<Vec<Cell>> {
        let stake_cell = collect_cells(&self.ckb.client, 1, SearchKey {
            script:               self.stake_lock.clone().into(),
            script_type:          ScriptType::Lock,
            filter:               Some(SearchKeyFilter {
                script: Some(self.xudt.clone().into()),
                ..Default::default()
            }),
            script_search_mode:   None,
            with_data:            Some(true),
            group_by_transaction: None,
        })
        .await?;
        Ok(stake_cell)
    }

    async fn build_first_stake_tx(&self) -> Result<TransactionView> {
        let mut inputs = vec![];

        // AT cells
        let token_amount = self.add_token_to_intpus(&mut inputs).await?;

        let outputs_data = self.first_stake_data(token_amount)?;

        let outputs = vec![
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
            // // delegate cell
            // CellOutput::new_builder()
            //     .lock(fake_lock.clone())
            //     .type_(Some(fake_type.clone()).pack())
            //     .build_exact_capacity(Capacity::bytes(outputs_data[2].len())?)?,
            // // withdraw AT cell
            // CellOutput::new_builder()
            //     .lock(fake_lock)
            //     .type_(Some(fake_type).pack())
            //     .build_exact_capacity(Capacity::bytes(outputs_data[3].len())?)?,
        ];

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
        let token_amount = self.add_token_to_intpus(&mut inputs).await?;

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
            stake_dep(&self.ckb.network_type),
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
            omni_eth_witness_placeholder().as_bytes(), // stake AT cell lock
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

    async fn add_token_to_intpus(&self, inputs: &mut Vec<CellInput>) -> Result<Amount> {
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

        let _delegate = self.delegate.as_ref().ok_or(CkbTxErr::Delegate)?;

        Ok(vec![
            // AT cell data
            wallet_amount.pack().as_bytes(),
            // stake AT cell data
            token_cell_data(
                self.stake.amount,
                stake_cell_data(
                    self.stake.is_increase,
                    self.stake.amount,
                    self.stake.inauguration_epoch,
                )
                .as_bytes(),
            ),
            // // delegate cell data
            // delegate_cell_data(
            //     delegate.threshold,
            //     delegate.maximum_delegators,
            //     delegate.dividend_ratio,
            // )
            // .as_bytes(),
            // // withdraw AT cell data
            // token_cell_data(0, withdraw_token_cell_data(None).as_bytes()),
        ])
    }

    fn update_stake_data(
        &self,
        wallet_amount: Amount,
        stake_data: Bytes,
    ) -> CkbTxResult<Vec<Bytes>> {
        let total_stake_amount = new_u128(&stake_data[..TOKEN_BYTES]);

        let mut stake_data = stake_data;
        let stake_data = StakeAtCellData::new_unchecked(stake_data.split_off(TOKEN_BYTES));
        let last_info =
            ElectAmountCaculator::last_stake_info(&stake_data.delta(), self.current_epoch);

        let actual_info = ElectAmountCaculator::new(
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
                stake_cell_data(
                    actual_info.is_increase,
                    actual_info.amount,
                    self.stake.inauguration_epoch,
                )
                .as_bytes(),
            ),
            // AT cell data
            actual_info.wallet_amount.pack().as_bytes(),
        ])
    }
}
