use std::collections::{HashMap, HashSet};

use anyhow::Result;
use async_trait::async_trait;
use ckb_types::{
    bytes::Bytes,
    core::{Capacity, TransactionBuilder, TransactionView},
    packed::{CellInput, CellOutput, Script},
    prelude::{Builder, Entity, Pack},
};

use common::traits::ckb_rpc_client::CkbRpc;
use common::traits::smt::DelegateSmtStorage;
use common::traits::tx_builder::IDelegateTxBuilder;
use common::types::axon_types::delegate::*;
use common::types::axon_types::withdraw::WithdrawAtCellData;
use common::types::ckb_rpc_client::Cell;
use common::types::tx_builder::{Amount, DelegateItem, Epoch, EthAddress, StakeTypeIds};
use common::utils::convert::*;

use crate::ckb::define::constants::{INAUGURATION, TOKEN_BYTES};
use crate::ckb::define::error::{CkbTxErr, CkbTxResult};
use crate::ckb::define::types::{
    DelegateAtCellData as TDelegateAtCellData, DelegateAtCellLockData as TDelegateAtCellLockData,
};
use crate::ckb::helper::{
    amount_calculator::*, token_cell_data, Checkpoint, Delegate, Metadata, OmniEth, Secp256k1, Tx,
    Withdraw, Xudt,
};

pub struct DelegateTxBuilder<'a, C: CkbRpc, D: DelegateSmtStorage> {
    ckb:                  &'a C,
    type_ids:             StakeTypeIds,
    delegate_smt_storage: D,
    current_epoch:        Epoch,
    delegator:            EthAddress,
    delegates:            Vec<DelegateItem>,
    delegate_lock:        Script,
    token_lock:           Script,
    withdraw_lock:        Script,
    xudt:                 Script,
}

#[async_trait]
impl<'a, C: CkbRpc, D: DelegateSmtStorage> IDelegateTxBuilder<'a, C, D>
    for DelegateTxBuilder<'a, C, D>
{
    fn new(
        ckb: &'a C,
        type_ids: StakeTypeIds,
        delegator: EthAddress,
        current_epoch: Epoch,
        delegates: Vec<DelegateItem>,
        delegate_smt_storage: D,
    ) -> Self {
        let delegate_lock = Delegate::lock(&type_ids.metadata_type_id, &delegator);
        let withdraw_lock = Withdraw::lock(&type_ids.metadata_type_id, &delegator);
        let token_lock = OmniEth::lock(&delegator);
        let xudt = Xudt::type_(&type_ids.xudt_owner.pack());

        Self {
            ckb,
            type_ids,
            delegate_smt_storage,
            current_epoch,
            delegator,
            delegates,
            delegate_lock,
            token_lock,
            withdraw_lock,
            xudt,
        }
    }

    async fn build_tx(self) -> Result<TransactionView> {
        for delegate in self.delegates.iter() {
            if delegate.inauguration_epoch > self.current_epoch + INAUGURATION {
                return Err(CkbTxErr::InaugurationEpoch {
                    expected: self.current_epoch,
                    found:    delegate.inauguration_epoch,
                }
                .into());
            }
        }

        let delegate_cell =
            Delegate::get_cell(self.ckb, self.delegate_lock.clone(), self.xudt.clone()).await?;

        if delegate_cell.is_none() {
            self.build_first_delegate_tx().await
        } else {
            self.build_update_delegate_tx(delegate_cell.unwrap().clone())
                .await
        }
    }
}

impl<'a, C: CkbRpc, D: DelegateSmtStorage> DelegateTxBuilder<'a, C, D> {
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

        let cell_deps = vec![OmniEth::lock_dep(), Secp256k1::lock_dep(), Xudt::type_dep()];

        let witnesses = vec![
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

    async fn build_update_delegate_tx(&self, delegate_cell: Cell) -> Result<TransactionView> {
        // delegate AT cell
        let mut inputs = vec![CellInput::new_builder()
            .previous_output(delegate_cell.out_point.into())
            .build()];

        let token_amount = self.add_token_to_intpus(&mut inputs).await?;

        let delegate_data = delegate_cell.output_data.unwrap().into_bytes();
        let outputs_data = self
            .update_delegate_data(token_amount, delegate_data)
            .await?;

        let outputs = vec![
            // delegate AT cell
            CellOutput::new_builder()
                .lock(self.delegate_lock.clone())
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
            Delegate::lock_dep(),
            Checkpoint::cell_dep(self.ckb, &self.type_ids.checkpoint_type_id).await?,
            Metadata::cell_dep(self.ckb, &self.type_ids.metadata_type_id).await?,
        ];

        let witnesses = vec![
            Delegate::witness(0u8).as_bytes(),         // delegate AT cell lock
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

    async fn add_token_to_intpus(&self, inputs: &mut Vec<CellInput>) -> Result<Amount> {
        let expected_amount = {
            let mut total_increase = 0;
            let mut total_decrease = 0;

            for item in self.delegates.iter() {
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

        let (token_cells, amount) = Xudt::collect(
            self.ckb,
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
        let withdraw_cell =
            Withdraw::get_cell(self.ckb, self.withdraw_lock.clone(), self.xudt.clone()).await?;

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
        log::info!(
            "[first delegate] delegator: {}, old wallet amount: {}",
            self.delegator.to_string(),
            wallet_amount
        );

        let mut total_delegate_amount = 0;
        let mut delegates = vec![];

        for item in self.delegates.iter() {
            if !item.is_increase {
                return Err(CkbTxErr::Increase(item.is_increase));
            }
            if item.staker == self.delegator {
                return Err(CkbTxErr::DelegateYourself);
            }
            total_delegate_amount += item.amount;

            log::info!(
                "[first delegate] delegator: {}, delegate to: {}, delegate amount: {}",
                self.delegator.to_string(),
                item.staker.to_string(),
                item.amount,
            );

            delegates.push(item.to_owned());
        }

        if wallet_amount < total_delegate_amount {
            return Err(CkbTxErr::ExceedWalletAmount(
                wallet_amount,
                total_delegate_amount,
            ));
        }
        wallet_amount -= total_delegate_amount;

        log::info!(
            "[first delegate] delegator: {}, new total delegate amount: {}, new wallet amount: {}",
            self.delegator.to_string(),
            total_delegate_amount,
            wallet_amount,
        );

        Ok(vec![
            // AT cell data
            wallet_amount.pack().as_bytes(),
            // delegate AT cell data
            token_cell_data(
                total_delegate_amount,
                DelegateAtCellData::from(TDelegateAtCellData {
                    lock: TDelegateAtCellLockData {
                        l2_address:      self.delegator.clone(),
                        delegator_infos: delegates,
                    },
                })
                .as_bytes(),
            ),
        ])
    }

    async fn update_delegate_data(
        &self,
        mut wallet_amount: Amount,
        delegate_data: Bytes,
    ) -> CkbTxResult<Vec<Bytes>> {
        let mut total_delegate_amount = new_u128(&delegate_data[..TOKEN_BYTES]);

        log::info!(
            "[update delegate] delegator: {}, old wallet amount: {}, old total delegate amount: {}",
            self.delegator.to_string(),
            wallet_amount,
            total_delegate_amount,
        );

        let mut delegate_data = delegate_data;
        let delegate_data = DelegateAtCellData::new_unchecked(delegate_data.split_off(TOKEN_BYTES));
        let last_delegates = self.collect_cell_delegates(
            &delegate_data.lock(),
            wallet_amount,
            total_delegate_amount,
        )?;

        let (mut updated_delegates, stakers) = self
            .process_new_delegates(
                &last_delegates,
                &mut wallet_amount,
                &mut total_delegate_amount,
            )
            .await?;

        // process rest delegate infos in delegate AT cell
        self.process_rest_delegates(
            last_delegates,
            &stakers,
            &mut wallet_amount,
            &mut total_delegate_amount,
            &mut updated_delegates,
        )?;

        log::info!(
            "[update delegate] delegator: {}, new wallet amount: {}, new total delegate amount: {}",
            self.delegator.to_string(),
            wallet_amount,
            total_delegate_amount,
        );

        let inner_delegate_data = delegate_data.lock();

        Ok(vec![
            // delegate AT cell data
            token_cell_data(
                total_delegate_amount,
                delegate_data
                    .as_builder()
                    .lock(
                        inner_delegate_data
                            .as_builder()
                            .delegator_infos({
                                DelegateInfoDeltas::new_builder()
                                    .extend(updated_delegates.into_iter().map(Into::into))
                                    .build()
                            })
                            .build(),
                    )
                    .build()
                    .as_bytes(),
            ),
            // AT cell data
            wallet_amount.pack().as_bytes(),
        ])
    }

    fn collect_cell_delegates(
        &self,
        cell_delegates: &DelegateAtCellLockData,
        wallet_amount: u128,
        total_delegate_amount: u128,
    ) -> CkbTxResult<HashMap<ckb_types::H160, DelegateItem>> {
        let mut total_redeem = 0;
        let mut total_add = 0;
        let mut last_delegates = HashMap::new();
        for delegate in cell_delegates.delegator_infos() {
            let delegate_item = DelegateItem::from(delegate);

            log::info!(
                "[update delegate] delegator: {}, old delegate info: {:?}",
                self.delegator.to_string(),
                delegate_item,
            );

            if delegate_item.staker == self.delegator {
                return Err(CkbTxErr::DelegateYourself);
            }

            if !delegate_item.is_increase
                && delegate_item.inauguration_epoch == self.current_epoch + INAUGURATION
            {
                total_redeem += delegate_item.amount;
            }
            if delegate_item.is_increase {
                total_add += delegate_item.amount;
            }

            last_delegates.insert(delegate_item.staker.clone(), delegate_item);
        }

        if total_redeem > total_delegate_amount {
            return Err(CkbTxErr::DelegateExceedTotalAmount {
                total_amount:  total_delegate_amount,
                redeem_amount: total_redeem,
            });
        }
        if total_add > wallet_amount {
            return Err(CkbTxErr::DelegateExceedWalletAmount {
                wallet_amount,
                delegate_amount: total_add,
            });
        }
        Ok(last_delegates)
    }

    async fn process_new_delegates(
        &self,
        last_delegates: &HashMap<ckb_types::H160, DelegateItem>,
        wallet_amount: &mut u128,
        total_delegate_amount: &mut u128,
    ) -> CkbTxResult<(Vec<DelegateItem>, HashSet<EthAddress>)> {
        let mut stakers = HashSet::new();
        let mut updated_delegates = Vec::new();

        for delegate in self.delegates.iter() {
            log::info!("[update delegate] new delegate info: {:?}", delegate);

            stakers.insert(delegate.staker.clone());

            if last_delegates.contains_key(&delegate.staker) {
                let last_delegate = last_delegates.get(&delegate.staker).unwrap();

                let actual_info = self
                    .update_delegate(
                        last_delegate,
                        delegate,
                        wallet_amount,
                        total_delegate_amount,
                    )
                    .await?;

                log::info!(
                    "[update delegate] exists in cell, actual delegate info: {}",
                    actual_info,
                );

                updated_delegates.push(DelegateItem {
                    staker:             delegate.staker.clone(),
                    is_increase:        actual_info.is_increase,
                    amount:             actual_info.amount,
                    inauguration_epoch: delegate.inauguration_epoch,
                });
            } else {
                log::info!(
                    "[update delegate] not exists in cell, actual delegate info: {:?}",
                    delegate
                );
                if delegate.is_increase {
                    if *wallet_amount < *total_delegate_amount {
                        return Err(CkbTxErr::ExceedWalletAmount(
                            *wallet_amount,
                            *total_delegate_amount,
                        ));
                    }
                    *wallet_amount -= delegate.amount;
                    *total_delegate_amount += delegate.amount;
                } else if self
                    .delegate_smt_storage
                    .get_amount(
                        self.current_epoch + INAUGURATION,
                        delegate.staker.0.into(),
                        self.delegator.0.into(),
                    )
                    .await
                    .unwrap()
                    .is_none()
                {
                    return Err(CkbTxErr::NeverDelegated(delegate.staker.clone()));
                }
                updated_delegates.push(delegate.to_owned());
            }
        }

        Ok((updated_delegates, stakers))
    }

    fn process_rest_delegates(
        &self,
        last_delegates: HashMap<ckb_types::H160, DelegateItem>,
        processed_stakers: &HashSet<EthAddress>,
        wallet_amount: &mut u128,
        total_delegate_amount: &mut u128,
        updated_delegates: &mut Vec<DelegateItem>,
    ) -> CkbTxResult<()> {
        for (_, delegate) in last_delegates.into_iter() {
            if !processed_stakers.contains(&delegate.staker) && delegate.amount != 0 {
                log::info!(
                    "[update delegate] rest delegates, delegate info: {:?}",
                    delegate
                );
                // expired
                if delegate.inauguration_epoch < self.current_epoch + INAUGURATION
                    && delegate.is_increase
                {
                    *wallet_amount += delegate.amount;
                    *total_delegate_amount -= delegate.amount;
                } else {
                    updated_delegates.push(delegate);
                }
            }
        }

        Ok(())
    }

    async fn update_delegate(
        &self,
        last_delegate: &DelegateItem,
        new_delegate: &DelegateItem,
        wallet_amount: &mut u128,
        total_delegate_amount: &mut u128,
    ) -> CkbTxResult<ActualAmount> {
        let smt_amount = self
            .delegate_smt_storage
            .get_amount(
                self.current_epoch + INAUGURATION,
                new_delegate.staker.0.into(),
                self.delegator.0.into(),
            )
            .await
            .unwrap()
            .unwrap_or(0);

        if !new_delegate.is_increase {
            if last_delegate.is_increase {
                if new_delegate.amount > last_delegate.amount + smt_amount {
                    return Err(CkbTxErr::RedeemDelegate(
                        new_delegate.staker.clone(),
                        self.delegator.clone(),
                        new_delegate.amount,
                        smt_amount,
                    ));
                }
            } else if new_delegate.amount + last_delegate.amount > smt_amount {
                return Err(CkbTxErr::RedeemDelegate(
                    new_delegate.staker.clone(),
                    self.delegator.clone(),
                    new_delegate.amount,
                    smt_amount,
                ));
            }
        }

        let actual_info = ElectAmountCalculator::new(
            *wallet_amount,
            *total_delegate_amount,
            ElectAmountCalculator::last_delegate_info(last_delegate, self.current_epoch),
            ElectItem::Delegate(new_delegate),
        )
        .calc_actual_amount()?;

        #[allow(clippy::nonminimal_bool)]
        if actual_info.is_increase || (!actual_info.is_increase && last_delegate.is_increase) {
            *wallet_amount = actual_info.wallet_amount;
            *total_delegate_amount = actual_info.total_amount;
        }

        Ok(actual_info)
    }
}
