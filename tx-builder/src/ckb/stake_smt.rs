use std::collections::{HashMap, HashSet};

use anyhow::Result;
use async_trait::async_trait;
use axon_types::{basic::Byte32, stake::StakeInfoDelta};
use ckb_types::{
    bytes::Bytes,
    core::{Capacity, TransactionBuilder, TransactionView},
    packed::{CellOutput, Script},
    prelude::{Entity, Pack},
    H256,
};

use common::types::{ckb_rpc_client::Cell, tx_builder::*};
use common::utils::convert::*;
use common::{
    traits::{ckb_rpc_client::CkbRpc, tx_builder::IStakeSmtTxBuilder},
    types::ckb_rpc_client::{ScriptType, SearchKey, SearchKeyFilter},
};

use crate::ckb::define::constants::{INAUGURATION, TOKEN_BYTES};
use crate::ckb::define::error::*;
use crate::ckb::utils::cell_data::*;

use super::utils::{
    cell_collector::collect_cells,
    script::{stake_lock, xudt_type},
};

pub struct StakeSmtTxBuilder<C: CkbRpc> {
    ckb:           CkbNetwork<C>,
    _kicker:       PrivateKey,
    xudt:          Script,
    current_epoch: Epoch,
    quorum:        u16,
    _stake_cells:  Vec<Cell>,
    stake_lock:    Script,
}

#[async_trait]
impl<C: CkbRpc> IStakeSmtTxBuilder<C> for StakeSmtTxBuilder<C> {
    fn new(
        ckb: CkbNetwork<C>,
        _kicker: PrivateKey,
        xudt_args: H256,
        current_epoch: Epoch,
        metadata_type_id: H256,
        staker: EthAddress,
        quorum: u16,
        _stake_cells: Vec<Cell>,
    ) -> Self {
        let stake_lock = stake_lock(&ckb.network_type, &metadata_type_id, &staker);
        let xudt = xudt_type(&ckb.network_type, &xudt_args.pack());

        Self {
            ckb,
            _kicker,
            xudt,
            current_epoch,
            quorum,
            _stake_cells,
            stake_lock,
        }
    }

    async fn build_tx(&self) -> Result<(TransactionView, NonTopStakers)> {
        // todo: get stake smt cell
        let inputs = vec![];

        let stake_data: HashMap<Staker, Bytes> = HashMap::new(); // todo
        let (root, statistics) = self.collect(stake_data)?;

        // todo: fill lock, type
        let fake_lock = Script::default();
        let fake_type = Script::default();
        let mut outputs = vec![
            // stake smt cell
            CellOutput::new_builder()
                .lock(fake_lock)
                .type_(Some(fake_type).pack())
                .build_exact_capacity(Capacity::bytes(root.len())?)?,
        ];

        // todo: modify inputs
        // remove expired stakes and non top stakers (not in smt) from inputs
        // add removed stakers (in smt) to inputs

        let mut outputs_data = vec![root];

        // todo: add removed stakers (in smt) to statistics.total_stake_amounts

        // insert stake AT cells and withdraw AT cells to outputs
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

        Ok((tx, statistics.non_top_stakers))
    }
}

struct Statistics {
    pub _expired_stakes:     HashSet<Staker>,
    pub non_top_stakers:     HashMap<Staker, InStakeSmt>,
    pub withdraw_amounts:    HashMap<Staker, Amount>,
    pub total_stake_amounts: HashMap<Staker, Amount>,
}

impl<C: CkbRpc> StakeSmtTxBuilder<C> {
    async fn get_stake_smt_cell(&self) -> Result<Vec<Cell>> {
        let stake_cell = collect_cells(&self.ckb.client, 1, SearchKey {
            script:               self.stake_lock.clone().into(),
            script_type:          ScriptType::Type,
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

    // todo: witness?
    fn fill_tx(
        &self,
        statistics: &Statistics,
        outputs: &mut Vec<CellOutput>,
        outputs_data: &mut Vec<Bytes>,
    ) -> Result<()> {
        let fake_lock = Script::default();
        let fake_type = Script::default();

        for (staker, total_stake_amount) in statistics.total_stake_amounts.iter() {
            if statistics.non_top_stakers.contains_key(staker) {
                let in_smt = statistics.non_top_stakers.get(staker).unwrap().to_owned();
                if !in_smt {
                    continue;
                }
            }

            let (stake_data, withdraw_data) = if statistics.withdraw_amounts.contains_key(staker) {
                let withdraw_amount = statistics.withdraw_amounts.get(staker).unwrap().to_owned();
                let old_withdraw_data = Bytes::new(); // todo: get withdraw AT cell

                (
                    token_cell_data(
                        total_stake_amount - withdraw_amount,
                        stake_cell_data(false, 0, 0).as_bytes(),
                    ),
                    Some(update_withdraw_data(
                        old_withdraw_data,
                        self.current_epoch,
                        withdraw_amount,
                    )),
                )
            } else {
                (
                    token_cell_data(
                        total_stake_amount.to_owned(),
                        stake_cell_data(false, 0, 0).as_bytes(),
                    ),
                    None,
                )
            };

            // stake AT cell
            outputs.push(
                CellOutput::new_builder()
                    .lock(fake_lock.clone())
                    .type_(Some(fake_type.clone()).pack())
                    .build_exact_capacity(Capacity::bytes(stake_data.len())?)?,
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

            outputs_data.push(stake_data);
        }
        Ok(())
    }

    fn collect(&self, stake_datas: HashMap<Staker, Bytes>) -> CkbTxResult<(Bytes, Statistics)> {
        let old_smt: HashMap<Staker, Amount> = HashMap::new(); // todo: get from smt
        let mut new_smt: HashMap<Staker, Amount> = HashMap::new();
        let mut expired_stakes: HashSet<Staker> = HashSet::new();
        let mut withdraw_amounts: HashMap<Staker, Amount> = HashMap::new();
        let mut total_stake_amounts: HashMap<Staker, Amount> = HashMap::new();

        for (staker, mut stake_data) in stake_datas.into_iter() {
            let total_stake_amount = new_u128(&stake_data[..TOKEN_BYTES]);
            let stake = stake_item(&StakeInfoDelta::new_unchecked(
                stake_data.split_off(TOKEN_BYTES),
            ));

            total_stake_amounts.insert(staker.clone(), total_stake_amount);

            if stake.inauguration_epoch < self.current_epoch + INAUGURATION {
                expired_stakes.insert(staker.clone());
                continue;
            }

            if old_smt.contains_key(&staker) {
                let origin_stake_amount = old_smt.get(&staker).unwrap().to_owned();
                if stake.is_increase {
                    new_smt.insert(staker.clone(), origin_stake_amount + stake.amount);
                } else {
                    let withdraw_amount = if origin_stake_amount < stake.amount {
                        origin_stake_amount
                    } else {
                        stake.amount
                    };
                    new_smt.insert(staker.clone(), origin_stake_amount - withdraw_amount);
                    withdraw_amounts.insert(staker.clone(), withdraw_amount);
                }
            } else {
                if !stake.is_increase {
                    return Err(CkbTxErr::Increase(stake.is_increase));
                }
                new_smt.insert(staker.clone(), stake.amount);
            }
        }

        for (staker, amount) in old_smt.iter() {
            if !new_smt.contains_key(staker) {
                new_smt.insert(staker.to_owned(), amount.to_owned());
            }
        }

        let non_top_stakers =
            self.remove_non_top_stakers(&old_smt, &mut new_smt, &mut withdraw_amounts);

        // todo: insert new_smt to stake smt

        // todo: new smt root
        let new_root = Byte32::default();

        Ok((stake_smt_cell_data(new_root).as_bytes(), Statistics {
            _expired_stakes: expired_stakes,
            non_top_stakers,
            withdraw_amounts,
            total_stake_amounts,
        }))
    }

    fn remove_non_top_stakers(
        &self,
        old_smt: &HashMap<Staker, Amount>,
        new_smt: &mut HashMap<Staker, Amount>,
        withdraw_amounts: &mut HashMap<Staker, Amount>,
    ) -> NonTopStakers {
        if new_smt.len() <= 3 * self.quorum as usize {
            return HashMap::default();
        }

        let mut all_stakes = new_smt
            .clone()
            .into_iter()
            .map(|(k, v)| (k, v))
            .collect::<Vec<(Staker, Amount)>>();
        all_stakes.sort_unstable_by_key(|v| v.1);

        let delete_count = all_stakes.len() - 3 * self.quorum as usize;
        let delegted_stakers = &all_stakes[..delete_count];
        let mut non_top_stakers: HashMap<Staker, InStakeSmt> = HashMap::new();

        delegted_stakers.iter().for_each(|item| {
            let item = item.to_owned();
            let staker = item.0;

            new_smt.remove(&staker);
            non_top_stakers.insert(staker.clone(), false);

            if old_smt.contains_key(&staker) {
                withdraw_amounts.insert(staker.clone(), old_smt.get(&staker).unwrap().to_owned());
                non_top_stakers.insert(staker, true);
                // todo remove from stake smt
            }
        });

        non_top_stakers
    }
}
