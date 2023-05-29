use std::collections::{HashMap, HashSet};

use anyhow::Result;
use async_trait::async_trait;
use axon_types::{
    basic::Byte32,
    delegate::{DelegateInfoDelta, DelegateInfoDeltas},
};
use ckb_sdk::rpc::ckb_indexer::Cell;
use ckb_types::{
    bytes::Bytes,
    core::{Capacity, TransactionBuilder, TransactionView},
    packed::{CellOutput, Script},
    prelude::{Entity, Pack},
};

use common::traits::tx_builder::IDelegateSmtTxBuilder;
use common::types::tx_builder::*;
use common::utils::convert::*;

use crate::ckb::define::config::{INAUGURATION, TOKEN_BYTES};
use crate::ckb::define::error::*;
use crate::ckb::utils::cell_data::*;

pub struct DelegateSmtTxBuilder {
    _kicker:         PrivateKey,
    current_epoch:   Epoch,
    _quorum:         u16,
    _delegate_cells: Vec<Cell>,
}

#[async_trait]
impl IDelegateSmtTxBuilder for DelegateSmtTxBuilder {
    fn new(
        _kicker: PrivateKey,
        current_epoch: Epoch,
        _quorum: u16,
        _delegate_cells: Vec<Cell>,
    ) -> Self {
        Self {
            _kicker,
            current_epoch,
            _quorum,
            _delegate_cells,
        }
    }

    async fn build_tx(&self) -> Result<(TransactionView, NonTopDelegators)> {
        // todo: get delegate smt cell
        let inputs = vec![];

        // todo
        let delegate_datas: HashMap<Staker, Bytes> = HashMap::new(); // todo
        let (root, statistics) = self.collect(delegate_datas)?;

        // todo: fill lock, type
        let fake_lock = Script::default();
        let fake_type = Script::default();
        let mut outputs = vec![
            // delegate smt cell
            CellOutput::new_builder()
                .lock(fake_lock)
                .type_(Some(fake_type).pack())
                .build_exact_capacity(Capacity::bytes(root.len())?)?,
        ];

        // todo: modify inputs
        // remove expired delegates and non top delegators (all delegates are not
        //   in smt) from inputs
        // add removed delegators (in smt) to inputs

        let mut outputs_data = vec![root];

        // todo: add removed delegates (in smt) to statistics.total_amounts

        // insert delegate AT cells and withdraw AT cells to outputs
        self.fill_tx(&statistics, &mut outputs, &mut outputs_data)?;

        // todo
        let cell_deps = vec![];

        // todo: balance tx, fill placeholder witnesses,
        let tx = TransactionBuilder::default()
            .inputs(inputs)
            .outputs(outputs)
            .outputs_data(outputs_data.pack())
            .cell_deps(cell_deps)
            .build();

        // todo: sign tx

        Ok((tx, HashMap::default()))
    }
}

struct Statistics {
    pub expired_delegates:  HashSet<Delegator>,
    pub withdraw_amounts:   HashMap<Delegator, Amount>,
    pub total_amounts:      HashMap<Delegator, Amount>,
    pub non_top_delegators: HashMap<Delegator, HashMap<Staker, InDelegateSmt>>,
    pub _delegate_infos:    HashMap<Delegator, HashMap<Staker, DelegateInfoDelta>>,
}

impl DelegateSmtTxBuilder {
    // todo: witness?
    fn fill_tx(
        &self,
        statistics: &Statistics,
        outputs: &mut Vec<CellOutput>,
        outputs_data: &mut Vec<Bytes>,
    ) -> Result<()> {
        let fake_lock = Script::default();
        let fake_type = Script::default();

        for (delegator, total_amount) in statistics.total_amounts.iter() {
            if statistics.expired_delegates.contains(delegator) {
                continue;
            }

            if statistics.non_top_delegators.contains_key(delegator) {
                let all_delegates_not_in_smt = statistics
                    .non_top_delegators
                    .get(delegator)
                    .unwrap()
                    .values()
                    .all(|in_smt| !*in_smt);
                if all_delegates_not_in_smt {
                    continue;
                }
            }

            // todo: fix
            let (delegate_data, withdraw_data) = if statistics
                .withdraw_amounts
                .contains_key(delegator)
            {
                let withdraw_amount = statistics
                    .withdraw_amounts
                    .get(delegator)
                    .unwrap()
                    .to_owned();
                let old_withdraw_data = Bytes::new(); // todo: get withdraw AT cell

                (
                    token_cell_data(
                        total_amount - withdraw_amount,
                        delegate_cell_data(&[]).as_bytes(),
                    ),
                    Some(update_withdraw_data(
                        old_withdraw_data,
                        self.current_epoch,
                        withdraw_amount,
                    )),
                )
            } else {
                (
                    token_cell_data(total_amount.to_owned(), delegate_cell_data(&[]).as_bytes()),
                    None,
                )
            };

            // delegate AT cell
            outputs.push(
                CellOutput::new_builder()
                    .lock(fake_lock.clone())
                    .type_(Some(fake_type.clone()).pack())
                    .build_exact_capacity(Capacity::bytes(delegate_data.len())?)?,
            );

            // withdraw AT cell
            if withdraw_data.is_some() {
                outputs.push(
                    CellOutput::new_builder()
                        .lock(fake_lock.clone())
                        .type_(Some(fake_type.clone()).pack())
                        .build_exact_capacity(Capacity::bytes(withdraw_data.unwrap().len())?)?,
                );
            }

            outputs_data.push(delegate_data);
        }

        Ok(())
    }

    fn collect(
        &self,
        delegate_datas: HashMap<Delegator, Bytes>,
    ) -> CkbTxResult<(Bytes, Statistics)> {
        let mut delegates: HashMap<Staker, Vec<(Delegator, DelegateItem)>> = HashMap::new();
        let mut delegate_infos: HashMap<Delegator, HashMap<Staker, DelegateInfoDelta>> =
            HashMap::new();
        let mut total_amounts: HashMap<Delegator, Amount> = HashMap::new();
        let mut expired_delegates: HashSet<Delegator> = HashSet::new();

        self.collect_cell_delegates(
            delegate_datas,
            &mut delegates,
            &mut delegate_infos,
            &mut total_amounts,
            &mut expired_delegates,
        );

        let mut non_top_delegators: HashMap<Delegator, HashMap<Staker, InStakeSmt>> =
            HashMap::new();
        let mut withdraw_amounts: HashMap<Delegator, HashMap<Staker, Amount>> = HashMap::new();

        for (staker, delegators) in delegates.into_iter() {
            let old_smt: HashMap<Delegator, Amount> = HashMap::new(); // todo: get from smt

            let mut new_smt = self.collect_updated_delegates(
                staker.clone(),
                delegators,
                &old_smt,
                &mut withdraw_amounts,
            )?;

            let maximum_delegators = 10; // todo: get from staker's delegate cell

            self.remove_non_top_stakers(
                staker,
                maximum_delegators,
                &old_smt,
                &mut new_smt,
                &mut withdraw_amounts,
                &mut non_top_delegators,
            );

            // todo: insert new_smt to delegate smt
        }

        // todo: new smt roots
        let new_roots = vec![(EthAddress::default(), Byte32::default())];

        Ok((delegate_smt_cell_data(new_roots).as_bytes(), Statistics {
            expired_delegates,
            non_top_delegators,
            withdraw_amounts: calc_withdraw_amounts(withdraw_amounts),
            total_amounts,
            _delegate_infos: delegate_infos,
        }))
    }

    fn collect_cell_delegates(
        &self,
        delegate_datas: HashMap<Delegator, Bytes>,
        delegates: &mut HashMap<Staker, Vec<(Delegator, DelegateItem)>>,
        delegate_infos: &mut HashMap<Delegator, HashMap<Staker, DelegateInfoDelta>>,
        total_amounts: &mut HashMap<Delegator, Amount>,
        expired_delegates: &mut HashSet<Delegator>,
    ) {
        for (delegator, mut delegate_data) in delegate_datas.into_iter() {
            let total_amount = new_u128(&delegate_data[..TOKEN_BYTES]);
            total_amounts.insert(delegator.clone(), total_amount);

            let delegator_infos =
                DelegateInfoDeltas::new_unchecked(delegate_data.split_off(TOKEN_BYTES));
            for info in delegator_infos.into_iter() {
                let item = delegate_item(&info);
                if item.inauguration_epoch < self.current_epoch + INAUGURATION {
                    expired_delegates.insert(delegator.clone());
                    break;
                }
                delegates
                    .entry(item.staker.clone())
                    .and_modify(|e| e.push((delegator.clone(), item.clone())))
                    .or_insert(vec![(delegator.clone(), item.clone())]);
                delegate_infos
                    .entry(item.staker.clone())
                    .and_modify(|e| {
                        e.insert(delegator.clone(), info.clone());
                    })
                    .or_insert_with(HashMap::new)
                    .insert(delegator.clone(), info.clone());
            }
        }
    }

    fn collect_updated_delegates(
        &self,
        staker: Staker,
        delegators: Vec<(Delegator, DelegateItem)>,
        old_smt: &HashMap<Delegator, Amount>,
        withdraw_amounts: &mut HashMap<Delegator, HashMap<Staker, Amount>>,
    ) -> CkbTxResult<HashMap<Delegator, Amount>> {
        let mut new_smt: HashMap<Delegator, Amount> = HashMap::new();

        for (delegator, delegate) in delegators.into_iter() {
            if old_smt.contains_key(&delegator) {
                let origin_amount = old_smt.get(&delegator).unwrap().to_owned();
                if delegate.is_increase {
                    new_smt.insert(delegator.clone(), origin_amount + delegate.amount);
                } else {
                    let withdraw_amount = if origin_amount < delegate.amount {
                        origin_amount
                    } else {
                        delegate.amount
                    };
                    new_smt.insert(delegator.clone(), origin_amount - withdraw_amount);
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
                    return Err(CkbTxErr::Increase(delegate.is_increase));
                }
                new_smt.insert(delegator.clone(), delegate.amount);
            }
        }

        for (delegator, amount) in old_smt.iter() {
            if !new_smt.contains_key(delegator) {
                new_smt.insert(delegator.to_owned(), amount.to_owned());
            }
        }

        Ok(new_smt)
    }

    fn remove_non_top_stakers(
        &self,
        staker: Staker,
        maximum_delegators: u32,
        old_smt: &HashMap<Delegator, Amount>,
        new_smt: &mut HashMap<Delegator, Amount>,
        withdraw_amounts: &mut HashMap<Delegator, HashMap<Staker, Amount>>,
        non_top_delegators: &mut HashMap<Delegator, HashMap<Staker, InStakeSmt>>,
    ) {
        if new_smt.len() <= maximum_delegators as usize {
            return;
        }

        let mut all_delegates = new_smt
            .clone()
            .into_iter()
            .map(|(k, v)| (k, v))
            .collect::<Vec<(Delegator, Amount)>>();
        all_delegates.sort_unstable_by_key(|v| v.1);

        let delete_count = all_delegates.len() - maximum_delegators as usize;
        let delegted_stakers = &all_delegates[..delete_count];

        delegted_stakers.iter().for_each(|item| {
            let item = item.to_owned();
            let delegator = item.0;
            let amount = item.1;

            new_smt.remove(&delegator);

            let mut in_smt = false;
            if old_smt.contains_key(&delegator) {
                in_smt = true;
                withdraw_amounts
                    .entry(delegator.clone())
                    .and_modify(|e| {
                        e.insert(staker.clone(), amount);
                    })
                    .or_insert_with(HashMap::new)
                    .insert(staker.clone(), amount);
            }

            non_top_delegators
                .entry(delegator)
                .and_modify(|e| {
                    e.insert(staker.clone(), in_smt);
                })
                .or_insert_with(HashMap::new)
                .insert(staker.clone(), in_smt);
        });
    }
}

fn calc_withdraw_amounts(
    withdraw_amounts: HashMap<Delegator, HashMap<Staker, Amount>>,
) -> HashMap<Delegator, Amount> {
    let mut total_withdraw_amounts: HashMap<Delegator, Amount> = HashMap::new();
    for item in withdraw_amounts.into_iter() {
        let delegator = item.0;
        let withdraw_map = item.1;

        let total_withdraw_amount = withdraw_map
            .values()
            .fold(0_u128, |acc, x| acc + x.to_owned());
        total_withdraw_amounts.insert(delegator.clone(), total_withdraw_amount);
    }
    total_withdraw_amounts
}
