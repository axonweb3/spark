use std::collections::{HashMap, HashSet};

use anyhow::Result;
use async_trait::async_trait;
use axon_types::delegate::*;
use ckb_types::{
    bytes::Bytes,
    core::{Capacity, TransactionBuilder, TransactionView},
    packed::{CellOutput, Script},
    prelude::{Builder, Entity, Pack},
};

use common::traits::tx_builder::IDelegateTxBuilder;
use common::types::tx_builder::{Address, DelegateItem, Epoch};
use common::utils::convert::*;

use crate::ckb::define::config::{INAUGURATION, TOKEN_BYTES};
use crate::ckb::define::error::{CkbTxErr, CkbTxResult};
use crate::ckb::utils::{calc_amount::*, cell_data::*};

pub struct DelegateTxBuilder {
    _delegator:    Address,
    current_epoch: Epoch,
    delegators:    Vec<DelegateItem>,
}

#[async_trait]
impl IDelegateTxBuilder for DelegateTxBuilder {
    fn new(_delegator: Address, current_epoch: Epoch, delegate_info: Vec<DelegateItem>) -> Self {
        Self {
            _delegator,
            current_epoch,
            delegators: delegate_info,
        }
    }

    async fn build_tx(&self) -> Result<TransactionView> {
        for delegate in self.delegators.iter() {
            if delegate.inauguration_epoch > self.current_epoch + 2 {
                return Err(CkbTxErr::InaugurationEpoch {
                    expected: self.current_epoch,
                    found:    delegate.inauguration_epoch,
                }
                .into());
            }
        }

        if self.is_first_delegate().await? {
            self.build_first_delegate_tx().await
        } else {
            self.build_update_delegate_tx().await
        }
    }
}

impl DelegateTxBuilder {
    async fn is_first_delegate(&self) -> CkbTxResult<bool> {
        // todo: search user's delegate AT cell
        Ok(true)
    }

    async fn build_first_delegate_tx(&self) -> Result<TransactionView> {
        // todo: collect AT cells
        let inputs = vec![];

        let wallet_data = vec![Bytes::default()]; // todo
        let outputs_data = self.first_delegate_data(&wallet_data)?;

        // todo: fill lock, type
        let fake_lock = Script::default();
        let fake_type = Script::default();
        let outputs = vec![
            // AT cell
            CellOutput::new_builder()
                .lock(fake_lock.clone())
                .type_(Some(fake_type.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[0].len())?)?,
            // delegate AT cell
            CellOutput::new_builder()
                .lock(fake_lock.clone())
                .type_(Some(fake_type.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[1].len())?)?,
            // withdraw AT cell
            CellOutput::new_builder()
                .lock(fake_lock)
                .type_(Some(fake_type).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[2].len())?)?,
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

    async fn build_update_delegate_tx(&self) -> Result<TransactionView> {
        // todo: collect AT cells
        // todo: get delegate AT cell
        let inputs = vec![];

        let wallet_data = vec![Bytes::default()]; // todo
        let delegate_data = Bytes::default(); // todo

        let outputs_data = self.update_delegate_data(&wallet_data, delegate_data)?;

        // todo: fill lock, type
        let fake_lock = Script::default();
        let fake_type = Script::default();
        let outputs = vec![
            // AT cell
            CellOutput::new_builder()
                .lock(fake_lock.clone())
                .type_(Some(fake_type.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[0].len())?)?,
            // delegate AT cell
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

    fn first_delegate_data(&self, wallet_data: &[Bytes]) -> CkbTxResult<Vec<Bytes>> {
        let mut total_amount = 0;
        for item in self.delegators.iter() {
            if !item.is_increase {
                return Err(CkbTxErr::Increase(item.is_increase));
            }
            total_amount += item.amount;
        }

        let mut wallet_amount = ElectAmountCaculator::calc_wallet_amount(wallet_data);
        if wallet_amount < total_amount {
            return Err(CkbTxErr::ExceedWalletAmount {
                wallet_amount,
                amount: total_amount,
            });
        }
        wallet_amount -= total_amount;

        Ok(vec![
            // AT cell data
            to_uint128(wallet_amount).as_bytes(),
            // delegate AT cell data
            token_cell_data(
                total_amount,
                delegate_token_cell_data(&self.delegators).as_bytes(),
            ),
            // withdraw AT cell data
            token_cell_data(0, withdraw_token_cell_data(None).as_bytes()),
        ])
    }

    fn update_delegate_data(
        &self,
        wallet_data: &[Bytes],
        delegate_data: Bytes,
    ) -> CkbTxResult<Vec<Bytes>> {
        let mut wallet_amount = ElectAmountCaculator::calc_wallet_amount(wallet_data);
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
            // AT cell data
            wallet_amount.pack().as_bytes(),
            // delegate AT cell data
            token_cell_data(total_amount, updated_delegates.build().as_bytes()),
        ])
    }

    fn process_new_delegates(
        &self,
        cell_delegates: &DelegateAtCellData,
        wallet_amount: &mut u128,
        total_amount: &mut u128,
    ) -> CkbTxResult<(DelegateInfoDeltasBuilder, HashSet<Address>)> {
        let mut last_delegates = HashMap::new();
        for delegate in cell_delegates.delegator_infos() {
            let last_staker = delegate.staker();
            last_delegates.insert(to_h160(last_staker), delegate);
        }

        let mut stakers = HashSet::new();
        let mut updated_delegates = DelegateInfoDeltas::new_builder();

        for delegate in self.delegators.iter() {
            stakers.insert(delegate.staker.clone());

            if last_delegates.contains_key(&delegate.staker) {
                let last_delegate_info = last_delegates.get(&delegate.staker).unwrap();

                if to_u128(last_delegate_info.amount()) == 0 {
                    continue;
                }

                let actual_info = self.update_delegate(
                    last_delegate_info,
                    delegate,
                    wallet_amount,
                    total_amount,
                )?;

                updated_delegates = updated_delegates.push(
                    (&DelegateItem {
                        staker:             delegate.staker.clone(),
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
                updated_delegates = updated_delegates.push(delegate.into());
            }
        }

        Ok((updated_delegates, stakers))
    }

    fn process_rest_delegates(
        &self,
        cell_delegates: &DelegateAtCellData,
        new_stakers: &HashSet<Address>,
        wallet_amount: &mut u128,
        total_amount: &mut u128,
        updated_delegates: DelegateInfoDeltasBuilder,
    ) -> CkbTxResult<DelegateInfoDeltasBuilder> {
        let mut updated_delegates = updated_delegates;

        for delegate in cell_delegates.delegator_infos() {
            let last_staker = delegate.staker();

            if !new_stakers.contains(&to_h160(last_staker)) && to_u128(delegate.amount()) != 0 {
                let delegate_item = delegate_item(&delegate);

                if delegate_item.inauguration_epoch < self.current_epoch + INAUGURATION {
                    process_expired_delegate(&delegate_item, wallet_amount, total_amount)?;
                } else {
                    updated_delegates = updated_delegates.push((&delegate_item).into());
                }
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
    ) -> CkbTxResult<ActualAmount> {
        let actual_info = ElectAmountCaculator::new(
            *wallet_amount,
            *total_amount,
            ElectAmountCaculator::last_delegate_info(last_delegate, self.current_epoch),
            ElectItem::Delegate(new_delegate),
        )
        .calc_actual_amount()?;

        *wallet_amount = actual_info.wallet_amount;
        *total_amount = actual_info.total_elect_amount;

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
) -> CkbTxResult<()> {
    if delegate.is_increase {
        if *wallet_amount < delegate.amount {
            return Err(CkbTxErr::ExceedWalletAmount {
                wallet_amount: *wallet_amount,
                amount:        delegate.amount,
            });
        }
        *wallet_amount += delegate.amount;
        *total_amount -= delegate.amount;
    }
    Ok(())
}
