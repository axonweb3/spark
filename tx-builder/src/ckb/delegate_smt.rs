use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use axon_types::{
    basic::Byte32,
    delegate::{DelegateArgs, DelegateAtCellData, DelegateCellData},
};
use ckb_types::{
    bytes::Bytes,
    core::{Capacity, TransactionBuilder, TransactionView},
    packed::{CellInput, CellOutput},
    prelude::{Entity, Pack},
};

use common::types::smt::{Delegator as SmtDelegator, Staker as SmtStaker};
use common::types::tx_builder::{
    Amount, CkbNetwork, DelegateItem, DelegateTypeIds, Delegator, Epoch, InDelegateSmt, InStakeSmt,
    NonTopDelegators, PrivateKey, Staker as TxStaker,
};
use common::types::{ckb_rpc_client::Cell, smt::UserAmount};
use common::utils::convert::{new_u128, to_usize};
use common::{
    traits::{ckb_rpc_client::CkbRpc, smt::DelegateSmtStorage, tx_builder::IDelegateSmtTxBuilder},
    types::ckb_rpc_client::{ScriptType, SearchKey},
};
use molecule::prelude::Builder;

use crate::ckb::define::{
    constants::{INAUGURATION, TOKEN_BYTES},
    error::CkbTxErr,
};
use crate::ckb::utils::cell_data::{
    delegate_cell_data, delegate_item, delegate_smt_cell_data, token_cell_data,
    update_withdraw_data,
};
use crate::ckb::utils::{
    cell_collector::{collect_cells, get_delegate_cell, get_unique_cell, get_withdraw_cell},
    cell_dep::{
        checkpoint_cell_dep, delegate_dep, metadata_cell_dep, omni_lock_dep, secp256k1_lock_dep,
        withdraw_lock_dep, xudt_type_dep,
    },
    omni::{omni_eth_address, omni_eth_witness_placeholder},
    script::{
        always_success_lock, delegate_lock, delegate_smt_type, omni_eth_lock, withdraw_lock,
        xudt_type,
    },
    tx::balance_tx,
};

pub struct DelegateSmtTxBuilder<C: CkbRpc, D: DelegateSmtStorage + Send + Sync> {
    ckb:                  CkbNetwork<C>,
    kicker:               PrivateKey,
    current_epoch:        Epoch,
    type_ids:             DelegateTypeIds,
    delegate_at_cells:    Vec<Cell>,
    delegate_smt_storage: D,
    cells:                HashMap<Delegator, Cell>,
}

#[async_trait]
impl<C: CkbRpc, D: DelegateSmtStorage + Send + Sync> IDelegateSmtTxBuilder<C, D>
    for DelegateSmtTxBuilder<C, D>
{
    fn new(
        ckb: CkbNetwork<C>,
        kicker: PrivateKey,
        current_epoch: Epoch,
        type_ids: DelegateTypeIds,
        delegate_at_cells: Vec<Cell>,
        delegate_smt_storage: D,
    ) -> Self {
        Self {
            ckb,
            kicker,
            current_epoch,
            type_ids,
            delegate_at_cells,
            delegate_smt_storage,
            cells: HashMap::new(),
        }
    }

    async fn build_tx(&mut self) -> Result<(TransactionView, NonTopDelegators)> {
        let delegate_lock = always_success_lock(&self.ckb.network_type); // todo: no one can delete delegate cell
        let delegate_type =
            delegate_smt_type(&self.ckb.network_type, &self.type_ids.delegate_smt_type_id);

        let delegate_smt_cell = get_unique_cell(&self.ckb.client, delegate_type.clone()).await?;

        let mut inputs = vec![
            // delegate smt cell
            CellInput::new_builder()
                .previous_output(delegate_smt_cell.out_point.clone().into())
                .build(),
        ];

        let (root, statistics) = self.collect().await?;

        let mut outputs = vec![
            // delegate smt cell
            CellOutput::new_builder()
                .lock(delegate_lock)
                .type_(Some(delegate_type).pack())
                .build_exact_capacity(Capacity::bytes(root.len())?)?,
        ];

        let mut outputs_data = vec![root];

        // insert delegate AT cells and withdraw AT cells to outputs
        self.fill_tx(&statistics, &mut inputs, &mut outputs, &mut outputs_data)
            .await?;

        // todo
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
            withdraw_lock_dep(&self.ckb.network_type),
        ];

        let witnesses = vec![
            omni_eth_witness_placeholder().as_bytes(), // Delegate AT cell lock
            omni_eth_witness_placeholder().as_bytes(), // Withdraw AT cell lock
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

        let kicker_addr = omni_eth_address(self.kicker.clone())?;
        let kicker_lock = omni_eth_lock(&self.ckb.network_type, &kicker_addr);
        let tx = balance_tx(&self.ckb.client, kicker_lock, tx).await?;

        // todo: sign tx

        Ok((tx, statistics.non_top_delegators))
    }
}

struct Statistics {
    pub withdraw_amounts:       HashMap<Delegator, Amount>,
    pub total_amounts:          HashMap<Delegator, Amount>,
    pub non_top_delegators:     HashMap<Delegator, HashMap<TxStaker, InDelegateSmt>>,
    pub non_top_delegate_items: HashMap<Delegator, Vec<DelegateItem>>,
}

impl<C: CkbRpc, D: DelegateSmtStorage + Send + Sync> DelegateSmtTxBuilder<C, D> {
    async fn fill_tx(
        &self,
        statistics: &Statistics,
        inputs: &mut Vec<CellInput>,
        outputs: &mut Vec<CellOutput>,
        outputs_data: &mut Vec<Bytes>,
    ) -> Result<()> {
        let delegate_lock = always_success_lock(&self.ckb.network_type); // todo: no one can delete delegate cell

        let xudt = xudt_type(&self.ckb.network_type, &self.type_ids.xudt_owner.pack());

        for (delegator, total_amount) in statistics.total_amounts.iter() {
            let withdraw_at_cell_lock = withdraw_lock(
                &self.ckb.network_type,
                &self.type_ids.metadata_type_id,
                delegator,
            );
            inputs.push(
                CellInput::new_builder()
                    .previous_output(self.cells[delegator].out_point.clone().into())
                    .build(),
            );

            let non_top_delegates = statistics.non_top_delegate_items.get(delegator).unwrap();

            let (delegate_data, withdraw_data) =
                if statistics.withdraw_amounts.contains_key(delegator) {
                    let withdraw_amount = statistics
                        .withdraw_amounts
                        .get(delegator)
                        .unwrap()
                        .to_owned();
                    let old_withdraw_cell = get_withdraw_cell(
                        &self.ckb.client,
                        withdraw_at_cell_lock.clone(),
                        xudt.clone(),
                    )
                    .await?
                    .unwrap();

                    inputs.push(
                        CellInput::new_builder()
                            .previous_output(old_withdraw_cell.out_point.clone().into())
                            .build(),
                    );

                    (
                        token_cell_data(
                            total_amount - withdraw_amount,
                            delegate_cell_data(non_top_delegates).as_bytes(),
                        ),
                        Some(update_withdraw_data(
                            old_withdraw_cell,
                            self.current_epoch + INAUGURATION,
                            withdraw_amount,
                        )),
                    )
                } else {
                    (
                        token_cell_data(
                            total_amount.to_owned(),
                            delegate_cell_data(non_top_delegates).as_bytes(),
                        ),
                        None,
                    )
                };

            // delegate AT cell
            outputs.push(
                CellOutput::new_builder()
                    .lock(delegate_lock.clone())
                    .type_(Some(xudt.clone()).pack())
                    .build_exact_capacity(Capacity::bytes(delegate_data.len())?)?,
            );
            outputs_data.push(delegate_data);

            // withdraw AT cell
            if withdraw_data.is_some() {
                outputs.push(
                    CellOutput::new_builder()
                        .lock(withdraw_at_cell_lock)
                        .type_(Some(xudt.clone()).pack())
                        .build_exact_capacity(Capacity::bytes(
                            withdraw_data.as_ref().unwrap().len(),
                        )?)?,
                );
                outputs_data.push(withdraw_data.unwrap());
            }
        }

        Ok(())
    }

    async fn collect(&mut self) -> Result<(Bytes, Statistics)> {
        let mut total_amounts = HashMap::new();
        let mut delegates = HashMap::new();
        self.collect_cell_delegates(&mut total_amounts, &mut delegates)?;

        let mut non_top_delegators = HashMap::new();
        let mut withdraw_amounts = HashMap::new();

        let mut roots = vec![];

        for (staker, delegators) in delegates.iter() {
            let smt_staker = SmtStaker::from_slice(staker.as_bytes());
            let old_smt = self
                .delegate_smt_storage
                .get_sub_leaves(self.current_epoch + INAUGURATION, smt_staker)
                .await?;

            let mut new_smt = old_smt.clone();

            self.collect_updated_delegates(
                staker.clone(),
                delegators.clone(),
                &mut new_smt,
                &mut withdraw_amounts,
            )?;

            self.remove_non_top_delegators(
                staker.clone(),
                &old_smt,
                &mut new_smt,
                &mut withdraw_amounts,
                &mut non_top_delegators,
                &mut total_amounts,
            )
            .await?;

            let new_delegators = new_smt
                .into_iter()
                .map(|(k, v)| UserAmount {
                    user:        k,
                    amount:      v,
                    is_increase: true,
                })
                .collect();

            self.delegate_smt_storage
                .insert(
                    self.current_epoch + INAUGURATION,
                    smt_staker,
                    new_delegators,
                )
                .await?;

            let root = Byte32::from_slice(
                self.delegate_smt_storage
                    .get_top_root(smt_staker)
                    .await?
                    .as_slice(),
            )
            .unwrap();
            roots.push((staker.to_owned(), root));
        }

        let mut non_top_delegate_items = HashMap::new();

        for (delegator, staker_info) in non_top_delegators.iter() {
            for (staker, in_smt) in staker_info.iter() {
                if !in_smt {
                    let delegate_item = delegates
                        .get(staker)
                        .unwrap()
                        .get(delegator)
                        .unwrap()
                        .to_owned();
                    non_top_delegate_items
                        .entry(delegator.clone())
                        .or_insert_with(Vec::new)
                        .push(delegate_item);
                    delegates.get_mut(staker).unwrap().remove(delegator);
                    if delegates.get(staker).unwrap().is_empty() {
                        delegates.remove(staker);
                    }
                }
            }
        }

        let mut delegator_delegates: HashMap<Delegator, Vec<DelegateItem>> = HashMap::new();
        for delegators in delegates.values() {
            for (delegator, delegate) in delegators.iter() {
                delegator_delegates
                    .entry(delegator.clone())
                    .and_modify(|e| e.push(delegate.clone()))
                    .or_insert(vec![delegate.clone()]);
            }
        }

        for delegator in non_top_delegate_items.clone().keys() {
            if !delegator_delegates.contains_key(delegator) {
                non_top_delegate_items.remove(delegator);
                total_amounts.remove(delegator);
            }
        }

        let aggregated_withdraw_amounts = withdraw_amounts
            .into_iter()
            .map(|(d, m)| {
                let total_withdraw_amount = m.values().fold(0_u128, |acc, x| acc + x.to_owned());
                (d, total_withdraw_amount)
            })
            .collect::<HashMap<Delegator, Amount>>();

        Ok((delegate_smt_cell_data(roots).as_bytes(), Statistics {
            non_top_delegators,
            withdraw_amounts: aggregated_withdraw_amounts,
            total_amounts,
            non_top_delegate_items,
        }))
    }

    fn collect_cell_delegates(
        &mut self,
        total_amounts: &mut HashMap<Delegator, Amount>,
        delegates: &mut HashMap<TxStaker, HashMap<Delegator, DelegateItem>>,
    ) -> Result<()> {
        for cell in self.delegate_at_cells.clone().into_iter() {
            let delegator = Delegator::from_slice(
                &DelegateArgs::new_unchecked(cell.output.lock.args.as_bytes().to_owned().into())
                    .delegator_addr()
                    .as_bytes(),
            )
            .unwrap();

            let mut cell_bytes = cell.output_data.clone().unwrap().into_bytes();
            let total_amount = new_u128(&cell_bytes[..TOKEN_BYTES]);

            let delegate = &DelegateAtCellData::new_unchecked(cell_bytes.split_off(TOKEN_BYTES));
            let delegator_infos = delegate.delegator_infos();
            let mut is_valid = false;

            for info in delegator_infos.into_iter() {
                let item = delegate_item(&info);
                if item.inauguration_epoch < self.current_epoch + INAUGURATION {
                    break;
                } else {
                    is_valid = true;
                    delegates
                        .entry(item.staker.clone())
                        .and_modify(|e| {
                            e.insert(delegator.clone(), item.clone());
                        })
                        .or_insert_with(HashMap::new)
                        .insert(delegator.clone(), item.clone());
                }
            }

            if is_valid {
                self.cells.insert(delegator.clone(), cell);
                total_amounts.insert(delegator.clone(), total_amount);
            }
        }

        Ok(())
    }

    fn collect_updated_delegates(
        &self,
        staker: TxStaker,
        delegators: HashMap<Delegator, DelegateItem>,
        new_smt: &mut HashMap<SmtDelegator, Amount>,
        withdraw_amounts: &mut HashMap<Delegator, HashMap<TxStaker, Amount>>,
    ) -> Result<()> {
        for (delegator, delegate) in delegators.into_iter() {
            let smt_delegator = SmtDelegator::from_slice(delegator.as_bytes());
            if new_smt.contains_key(&smt_delegator) {
                let origin_amount = new_smt.get(&smt_delegator).unwrap().to_owned();
                if delegate.is_increase {
                    new_smt.insert(smt_delegator, origin_amount + delegate.amount);
                } else {
                    let withdraw_amount = if origin_amount < delegate.amount {
                        origin_amount
                    } else {
                        delegate.amount
                    };
                    new_smt.insert(smt_delegator, origin_amount - withdraw_amount);
                    withdraw_amounts
                        .entry(delegator.clone())
                        .and_modify(|e| {
                            e.insert(staker.clone(), withdraw_amount);
                        })
                        .or_insert_with(HashMap::new)
                        .insert(staker.clone(), withdraw_amount);
                }
            } else {
                if !delegate.is_increase {
                    return Err(CkbTxErr::Increase(delegate.is_increase).into());
                }
                new_smt.insert(smt_delegator, delegate.amount);
            }
        }

        Ok(())
    }

    async fn remove_non_top_delegators(
        &mut self,
        staker: TxStaker,
        old_smt: &HashMap<SmtDelegator, Amount>,
        new_smt: &mut HashMap<SmtDelegator, Amount>,
        withdraw_amounts: &mut HashMap<Delegator, HashMap<TxStaker, Amount>>,
        non_top_delegators: &mut HashMap<Delegator, HashMap<TxStaker, InStakeSmt>>,
        total_amounts: &mut HashMap<Delegator, Amount>,
    ) -> Result<()> {
        let xudt = xudt_type(&self.ckb.network_type, &self.type_ids.xudt_owner.pack());
        let delegate_requirement_cell_lock = omni_eth_lock(&self.ckb.network_type, &staker);

        let delegate_requirement_cells = collect_cells(&self.ckb.client, 1, SearchKey {
            script:               delegate_requirement_cell_lock.into(),
            script_type:          ScriptType::Lock,
            filter:               None,
            script_search_mode:   None,
            with_data:            None,
            group_by_transaction: None,
        })
        .await?;

        let delegate_cell_bytes = delegate_requirement_cells[0]
            .to_owned()
            .output_data
            .unwrap()
            .into_bytes();
        let delegate_cell_info = DelegateCellData::new_unchecked(delegate_cell_bytes);
        let max_delegator_size = to_usize(
            delegate_cell_info
                .delegate_requirement()
                .max_delegator_size(),
        );

        if new_smt.len() <= max_delegator_size {
            return Ok(());
        }

        let mut all_delegates = new_smt
            .clone()
            .into_iter()
            .map(|(k, v)| (k, v))
            .collect::<Vec<(SmtDelegator, Amount)>>();

        all_delegates.sort_unstable_by_key(|v| v.1);

        let delete_count = all_delegates.len() - max_delegator_size;
        let deleted_delegators = &all_delegates[..delete_count];

        for (delegator, amount) in deleted_delegators {
            new_smt.remove(delegator);

            let tx_delegator = Delegator::from_slice(delegator.as_bytes()).unwrap();

            let mut in_smt = false;
            if old_smt.contains_key(delegator) {
                in_smt = true;
                withdraw_amounts
                    .entry(tx_delegator.clone())
                    .and_modify(|e| {
                        e.insert(staker.clone(), *amount);
                    })
                    .or_insert_with(HashMap::new)
                    .insert(staker.clone(), *amount);
                if !total_amounts.contains_key(&tx_delegator) {
                    let delegate_cell_lock = delegate_lock(
                        &self.ckb.network_type,
                        &self.type_ids.metadata_type_id,
                        &tx_delegator,
                    );

                    let cell =
                        get_delegate_cell(&self.ckb.client, delegate_cell_lock, xudt.clone())
                            .await?
                            .unwrap();
                    let cell_bytes = cell.output_data.clone().unwrap().into_bytes();
                    let total_delegate_amount = new_u128(&cell_bytes[..TOKEN_BYTES]);

                    self.cells.insert(staker.clone(), cell);

                    total_amounts.insert(tx_delegator.clone(), total_delegate_amount);
                }
            }

            non_top_delegators
                .entry(tx_delegator)
                .and_modify(|e| {
                    e.insert(staker.clone(), in_smt);
                })
                .or_insert_with(HashMap::new)
                .insert(staker.clone(), in_smt);
        }

        Ok(())
    }
}
