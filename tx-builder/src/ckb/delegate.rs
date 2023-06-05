use std::collections::{HashMap, HashSet};

use anyhow::Result;
use async_trait::async_trait;
use axon_types::delegate::*;
use axon_types::withdraw::WithdrawAtCellData;
use ckb_types::{
    bytes::Bytes,
    core::{Capacity, TransactionBuilder, TransactionView},
    packed::{CellInput, CellOutput, Script},
    prelude::{Builder, Entity, Pack},
};

use common::traits::ckb_rpc_client::CkbRpc;
use common::traits::tx_builder::IDelegateTxBuilder;
use common::types::ckb_rpc_client::Cell;
use common::types::tx_builder::{
    Amount, CkbNetwork, DelegateItem, Epoch, EthAddress, StakeTypeIds,
};
use common::utils::convert::*;

use crate::ckb::define::constants::{INAUGURATION, TOKEN_BYTES};
use crate::ckb::define::error::{CkbTxErr, CkbTxResult};
use crate::ckb::utils::{
    calc_amount::*,
    cell_collector::{collect_xudt, get_delegate_cell, get_withdraw_cell},
    cell_data::*,
    cell_dep::*,
    omni::*,
    script::*,
    tx::balance_tx,
};

pub struct DelegateTxBuilder<C: CkbRpc> {
    ckb:           CkbNetwork<C>,
    type_ids:      StakeTypeIds,
    current_epoch: Epoch,
    delegators:    Vec<DelegateItem>,
    delegate_lock: Script,
    token_lock:    Script,
    withdraw_lock: Script,
    xudt:          Script,
}

#[async_trait]
impl<C: CkbRpc> IDelegateTxBuilder<C> for DelegateTxBuilder<C> {
    fn new(
        ckb: CkbNetwork<C>,
        type_ids: StakeTypeIds,
        delegator: EthAddress,
        current_epoch: Epoch,
        delegators: Vec<DelegateItem>,
    ) -> Self {
        let delegate_lock =
            delegate_lock(&ckb.network_type, &type_ids.metadata_type_id, &delegator);
        let withdraw_lock = always_success_lock(&ckb.network_type); // todo
        let token_lock = omni_eth_lock(&ckb.network_type, &delegator);
        let xudt = xudt_type(&ckb.network_type, &type_ids.xudt_owner.pack());

        Self {
            ckb,
            type_ids,
            current_epoch,
            delegators,
            delegate_lock,
            token_lock,
            withdraw_lock,
            xudt,
        }
    }

    async fn build_tx(&self) -> Result<TransactionView> {
        for delegate in self.delegators.iter() {
            if delegate.inauguration_epoch > self.current_epoch + INAUGURATION {
                return Err(CkbTxErr::InaugurationEpoch {
                    expected: self.current_epoch,
                    found:    delegate.inauguration_epoch,
                }
                .into());
            }
        }

        let delegate_cell = get_delegate_cell(
            &self.ckb.client,
            self.delegate_lock.clone(),
            self.xudt.clone(),
        )
        .await?;

        if delegate_cell.is_none() {
            self.build_first_delegate_tx().await
        } else {
            self.build_update_delegate_tx(delegate_cell.unwrap().clone())
                .await
        }
    }
}

impl<C: CkbRpc> DelegateTxBuilder<C> {
    async fn build_first_delegate_tx(&self) -> Result<TransactionView> {
        let mut inputs = vec![];

        // AT cells
        let token_amount = self.add_token_to_intpus(&mut inputs).await?;

        let mut outputs_data = self.first_delegate_data(token_amount)?;

        let mut outputs = vec![
            // AT cell
            CellOutput::new_builder()
                .lock(self.token_lock.clone())
                .type_(Some(self.xudt.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[0].len())?)?,
            // delegate AT cell
            CellOutput::new_builder()
                .lock(self.delegate_lock.clone())
                .type_(Some(self.xudt.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[1].len())?)?,
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

    async fn build_update_delegate_tx(&self, delegate_cell: Cell) -> Result<TransactionView> {
        // delegate AT cell
        let mut inputs = vec![CellInput::new_builder()
            .previous_output(delegate_cell.out_point.into())
            .build()];

        let token_amount = self.add_token_to_intpus(&mut inputs).await?;

        let delegate_data = delegate_cell.output_data.unwrap().into_bytes();
        let outputs_data = self.update_delegate_data(token_amount, delegate_data)?;

        let outputs = vec![
            // delegate AT cell
            CellOutput::new_builder()
                .lock(self.delegate_lock.clone())
                .type_(Some(self.xudt.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[1].len())?)?,
            // AT cell
            CellOutput::new_builder()
                .lock(self.token_lock.clone())
                .type_(Some(self.xudt.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[0].len())?)?,
        ];

        let cell_deps = vec![
            omni_lock_dep(&self.ckb.network_type),
            secp256k1_lock_dep(&self.ckb.network_type),
            xudt_type_dep(&self.ckb.network_type),
            delegate_dep(&self.ckb.network_type),
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
            omni_eth_witness_placeholder().as_bytes(), // delegate AT cell lock
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
        let expected_amount = {
            let mut total_increase = 0;
            let mut total_decrease = 0;

            for item in self.delegators.iter() {
                if item.is_increase {
                    total_increase += item.amount;
                } else {
                    total_decrease += item.amount;
                }
            }

            if total_increase <= total_decrease {
                1
            } else {
                total_increase - total_decrease
            }
        };

        let (token_cells, amount) = collect_xudt(
            &self.ckb.client,
            self.token_lock.clone(),
            self.xudt.clone(),
            expected_amount,
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

    fn first_delegate_data(&self, mut wallet_amount: Amount) -> CkbTxResult<Vec<Bytes>> {
        let mut total_amount = 0;
        let mut delegates = vec![];
        for item in self.delegators.iter() {
            if !item.is_increase {
                return Err(CkbTxErr::Increase(item.is_increase));
            }
            total_amount += item.amount;

            let mut item = item.to_owned();
            item.total_amount = item.amount;
            delegates.push(item);
        }

        if wallet_amount < total_amount {
            return Err(CkbTxErr::ExceedWalletAmount {
                wallet_amount,
                amount: total_amount,
            });
        }
        wallet_amount -= total_amount;

        Ok(vec![
            // AT cell data
            wallet_amount.pack().as_bytes(),
            // delegate AT cell data
            token_cell_data(total_amount, delegate_cell_data(&delegates).as_bytes()),
        ])
    }

    fn update_delegate_data(
        &self,
        mut wallet_amount: Amount,
        delegate_data: Bytes,
    ) -> CkbTxResult<Vec<Bytes>> {
        let mut total_amount = new_u128(&delegate_data[..TOKEN_BYTES]);

        let mut delegate_data = delegate_data;
        let cell_delegates =
            DelegateAtCellData::new_unchecked(delegate_data.split_off(TOKEN_BYTES));

        let (updated_delegates, new_stakers) =
            self.process_new_delegates(&cell_delegates, &mut wallet_amount, &mut total_amount)?;

        // process rest delegate infos in delegate AT cell
        let updated_delegates = self.process_rest_delegates(
            &cell_delegates,
            &new_stakers,
            &mut wallet_amount,
            &mut total_amount,
            updated_delegates,
        )?;

        Ok(vec![
            // delegate AT cell data
            token_cell_data(total_amount, updated_delegates.build().as_bytes()),
            // AT cell data
            wallet_amount.pack().as_bytes(),
        ])
    }

    fn process_new_delegates(
        &self,
        cell_delegates: &DelegateAtCellData,
        wallet_amount: &mut u128,
        total_amount: &mut u128,
    ) -> CkbTxResult<(DelegateInfoDeltasBuilder, HashSet<EthAddress>)> {
        let mut last_delegates = HashMap::new();
        for delegate in cell_delegates.delegator_infos() {
            last_delegates.insert(to_h160(&delegate.staker()), delegate);
        }

        let mut stakers = HashSet::new();
        let mut updated_delegates = DelegateInfoDeltas::new_builder();

        for delegate in self.delegators.iter() {
            stakers.insert(delegate.staker.clone());

            if last_delegates.contains_key(&delegate.staker) {
                let last_delegate_info = last_delegates.get(&delegate.staker).unwrap();

                if to_u128(&last_delegate_info.amount()) == 0 {
                    continue;
                }

                let actual_info = self.update_delegate(
                    last_delegate_info,
                    delegate,
                    wallet_amount,
                    total_amount,
                    to_u128(&last_delegate_info.total_amount()),
                )?;

                updated_delegates = updated_delegates.push(
                    (&DelegateItem {
                        staker:             delegate.staker.clone(),
                        total_amount:       actual_info.total_elect_amount,
                        is_increase:        actual_info.is_increase,
                        amount:             actual_info.amount,
                        inauguration_epoch: delegate.inauguration_epoch,
                    })
                        .into(),
                );
            } else {
                if delegate.is_increase {
                    process_new_delegate(delegate.amount, wallet_amount, total_amount)?;
                }
                let mut delegate = delegate.to_owned();
                delegate.total_amount = delegate.amount;
                updated_delegates = updated_delegates.push((&delegate).into());
            }
        }

        Ok((updated_delegates, stakers))
    }

    fn process_rest_delegates(
        &self,
        cell_delegates: &DelegateAtCellData,
        new_stakers: &HashSet<EthAddress>,
        wallet_amount: &mut u128,
        total_amount: &mut u128,
        updated_delegates: DelegateInfoDeltasBuilder,
    ) -> CkbTxResult<DelegateInfoDeltasBuilder> {
        let mut updated_delegates = updated_delegates;

        for delegate in cell_delegates.delegator_infos() {
            let delta = delegate_item(&delegate);
            if !new_stakers.contains(&delta.staker)
                && (delta.total_amount != 0 || delta.amount != 0)
            {
                let delta = if delta.inauguration_epoch < self.current_epoch + INAUGURATION {
                    let total_staker_amount = process_expired_delegate(
                        &delta,
                        wallet_amount,
                        total_amount,
                        delta.total_amount,
                    )?;
                    delegate
                        .as_builder()
                        .total_amount(to_uint128(total_staker_amount))
                        .build()
                } else {
                    delegate
                };
                updated_delegates = updated_delegates.push(delta);
            }
        }

        Ok(updated_delegates)
    }

    fn update_delegate(
        &self,
        last_delegate: &DelegateInfoDelta,
        new_delegate: &DelegateItem,
        wallet_amount: &mut u128,
        total_amount: &mut u128,
        total_staker_amount: u128,
    ) -> CkbTxResult<ActualAmount> {
        let actual_info = ElectAmountCaculator::new(
            *wallet_amount,
            total_staker_amount,
            ElectAmountCaculator::last_delegate_info(last_delegate, self.current_epoch),
            ElectItem::Delegate(new_delegate),
        )
        .calc_actual_amount()?;

        *wallet_amount = actual_info.wallet_amount;
        *total_amount = if actual_info.is_increase {
            *total_amount + actual_info.total_elect_amount
        } else {
            *total_amount - actual_info.total_elect_amount
        };

        Ok(actual_info)
    }
}

fn process_new_delegate(
    amount: u128,
    wallet_amount: &mut u128,
    total_amount: &mut u128,
) -> CkbTxResult<()> {
    if *wallet_amount < amount {
        return Err(CkbTxErr::ExceedWalletAmount {
            wallet_amount: *wallet_amount,
            amount,
        });
    }
    *wallet_amount -= amount;
    *total_amount += amount;

    Ok(())
}

fn process_expired_delegate(
    delegate: &DelegateItem,
    wallet_amount: &mut u128,
    total_amount: &mut u128,
    mut total_staker_amount: u128,
) -> CkbTxResult<Amount> {
    if delegate.is_increase {
        if total_staker_amount < delegate.amount {
            return Err(CkbTxErr::ExceedTotalAmount {
                total_amount: total_staker_amount,
                new_amount:   delegate.amount,
            });
        }
        *wallet_amount += delegate.amount;
        *total_amount -= delegate.amount;
        total_staker_amount -= delegate.amount;
    }
    Ok(total_staker_amount)
}
