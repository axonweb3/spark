use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use axon_types::stake::StakeAtCellData;
use ckb_types::{
    bytes::Bytes,
    core::{Capacity, TransactionBuilder, TransactionView},
    packed::{CellInput, CellOutput},
    prelude::{Entity, Pack},
};

use common::types::{
    ckb_rpc_client::SearchKeyFilter,
    tx_builder::{StakeSmtTypeIds, Staker as TxStaker},
};
use common::{
    traits::smt::StakeSmtStorage,
    types::{
        ckb_rpc_client::Cell,
        tx_builder::{Amount, CkbNetwork, Epoch, InStakeSmt, NonTopStakers, PrivateKey},
    },
};
use common::{
    traits::{ckb_rpc_client::CkbRpc, tx_builder::IStakeSmtTxBuilder},
    types::ckb_rpc_client::{ScriptType, SearchKey},
};
use common::{
    types::smt::{Staker as SmtStaker, UserAmount},
    utils::convert::new_u128,
};
use molecule::prelude::Builder;

use crate::ckb::define::{
    constants::{INAUGURATION, TOKEN_BYTES},
    error::CkbTxErr,
};

use crate::ckb::utils::{
    cell_collector::{collect_cells, get_stake_cell, get_withdraw_cell},
    cell_data::{
        stake_cell_data, stake_item, stake_smt_cell_data, token_cell_data, update_withdraw_data,
    },
    cell_dep::{
        checkpoint_cell_dep, metadata_cell_dep, omni_lock_dep, secp256k1_lock_dep,
        withdraw_lock_dep, xudt_type_dep,
    },
    omni::{omni_eth_address, omni_eth_witness_placeholder},
    script::{always_success_lock, omni_eth_lock, stake_lock, stake_smt_type, xudt_type},
    tx::balance_tx,
};

use super::utils::cell_dep::stake_dep;

pub struct StakeSmtTxBuilder<C: CkbRpc, S: StakeSmtStorage + Send + Sync> {
    ckb:               CkbNetwork<C>,
    kicker:            PrivateKey,
    current_epoch:     Epoch,
    quorum:            u16,
    stake_cells:       Vec<Cell>,
    stake_smt_storage: S,
    type_ids:          StakeSmtTypeIds,
}

#[async_trait]
impl<C: CkbRpc, S: StakeSmtStorage + Send + Sync> IStakeSmtTxBuilder<C, S>
    for StakeSmtTxBuilder<C, S>
{
    fn new(
        ckb: CkbNetwork<C>,
        kicker: PrivateKey,
        current_epoch: Epoch,
        type_ids: StakeSmtTypeIds,
        quorum: u16,
        stake_cells: Vec<Cell>,
        stake_smt_storage: S,
    ) -> Self {
        Self {
            ckb,
            kicker,
            current_epoch,
            quorum,
            stake_cells,
            stake_smt_storage,
            type_ids,
        }
    }

    async fn build_tx(&self) -> Result<(TransactionView, NonTopStakers)> {
        let stake_lock = always_success_lock(&self.ckb.network_type); // todo: stake smt lock
        let stake_type = stake_smt_type(&self.ckb.network_type, &self.type_ids.metadata_type_id);

        let stake_smt_cells = collect_cells(&self.ckb.client, 1, SearchKey {
            script:               stake_lock.clone().into(),
            script_type:          ScriptType::Lock,
            filter:               Some(SearchKeyFilter {
                script: Some(stake_type.clone().into()),
                ..Default::default()
            }),
            script_search_mode:   None,
            with_data:            Some(true),
            group_by_transaction: None,
        })
        .await?;

        if stake_smt_cells.len() != 1 {
            return Err(CkbTxErr::SmtCellNum(stake_smt_cells.len()).into());
        }

        let mut inputs = vec![
            // stake smt cell
            CellInput::new_builder()
                .previous_output(stake_smt_cells[0].out_point.clone().into())
                .build(),
        ];

        let (root, cells, statistics) = self.collect().await?;

        let mut outputs = vec![
            // stake smt cell
            CellOutput::new_builder()
                .lock(stake_lock.clone())
                .type_(Some(stake_type).pack())
                .build_exact_capacity(Capacity::bytes(root.len())?)?,
        ];

        let mut outputs_data = vec![root];

        // insert stake AT cells and withdraw AT cells to outputs
        self.fill_tx(
            &statistics,
            &cells,
            &mut inputs,
            &mut outputs,
            &mut outputs_data,
        )
        .await?;

        let cell_deps = vec![
            omni_lock_dep(&self.ckb.network_type),
            secp256k1_lock_dep(&self.ckb.network_type),
            xudt_type_dep(&self.ckb.network_type),
            withdraw_lock_dep(&self.ckb.network_type),
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
            stake_dep(&self.ckb.network_type),
        ];

        let witnesses = vec![
            omni_eth_witness_placeholder().as_bytes(), // Stake smt cell lock
            omni_eth_witness_placeholder().as_bytes(), // Withdraw AT cell lock
            omni_eth_witness_placeholder().as_bytes(), // Stake AT cell lock, may not be needed
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

        Ok((tx, statistics.non_top_stakers))
    }
}

struct Statistics {
    pub non_top_stakers:     HashMap<TxStaker, InStakeSmt>,
    pub withdraw_amounts:    HashMap<TxStaker, Amount>,
    pub total_stake_amounts: HashMap<TxStaker, Amount>,
}

impl<C: CkbRpc, S: StakeSmtStorage + Send + Sync> StakeSmtTxBuilder<C, S> {
    // todo: witness?
    async fn fill_tx(
        &self,
        statistics: &Statistics,
        cells: &HashMap<TxStaker, Cell>,
        inputs: &mut Vec<CellInput>,
        outputs: &mut Vec<CellOutput>,
        outputs_data: &mut Vec<Bytes>,
    ) -> Result<()> {

        let xudt = xudt_type(&self.ckb.network_type, &self.type_ids.xudt_owner.pack());
        for (staker, total_stake_amount) in statistics.total_stake_amounts.iter() {
            let withdraw_lock = always_success_lock(&self.ckb.network_type); // todo: withdraw lock

            inputs.push(
                CellInput::new_builder()
                    .previous_output(cells[staker].out_point.clone().into())
                    .build(),
            );

            let (stake_data, withdraw_data) = if statistics.withdraw_amounts.contains_key(staker) {
                let withdraw_amount = statistics.withdraw_amounts.get(staker).unwrap().to_owned();
                let old_withdraw_data =
                    get_withdraw_cell(&self.ckb.client, withdraw_lock.clone(), xudt.clone())
                        .await?
                        .unwrap();

                inputs.push(
                    CellInput::new_builder()
                        .previous_output(old_withdraw_data.out_point.clone().into())
                        .build(),
                );

                (
                    token_cell_data(
                        total_stake_amount - withdraw_amount,
                        stake_cell_data(false, 0, 0).as_bytes(),
                    ),
                    Some(update_withdraw_data(
                        old_withdraw_data.output_data.clone().unwrap().into_bytes(),
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

            let stake_lock = stake_lock(
                &self.ckb.network_type,
                &self.type_ids.metadata_type_id,
                staker,
            );

            // stake AT cell
            outputs.push(
                CellOutput::new_builder()
                    .lock(stake_lock.clone())
                    .type_(Some(xudt.clone()).pack())
                    .build_exact_capacity(Capacity::bytes(stake_data.len())?)?,
            );
            outputs_data.push(stake_data);

            // withdraw AT cell
            if withdraw_data.is_some() {
                let withdraw_lock = always_success_lock(&self.ckb.network_type); // todo: withdraw lock

                outputs.push(
                    CellOutput::new_builder()
                        .lock(withdraw_lock.clone())
                        .type_(Some(xudt.clone()).pack())
                        .build_exact_capacity(Capacity::bytes(
                            withdraw_data.clone().unwrap().len(),
                        )?)?,
                );
                outputs_data.push(withdraw_data.unwrap())
            }
        }
        Ok(())
    }

    async fn collect(&self) -> Result<(Bytes, HashMap<TxStaker, Cell>, Statistics)> {
        let old_smt = self
            .stake_smt_storage
            .get_sub_leaves(self.current_epoch + 2)
            .await?;

        let xudt = xudt_type(&self.ckb.network_type, &self.type_ids.xudt_owner.pack());

        let mut new_smt = old_smt.clone();
        let mut withdraw_amounts = HashMap::new();
        let mut total_stake_amounts = HashMap::new();
        let mut cells = HashMap::new();

        for cell in self.stake_cells.clone().into_iter() {
            let mut cell_bytes = cell.output_data.clone().unwrap().into_bytes();
            let staker = TxStaker::from_slice(&cell.output.lock.args.as_bytes()[32..])?;

            let total_stake_amount = new_u128(&cell_bytes[..TOKEN_BYTES]);
            let stake = &StakeAtCellData::new_unchecked(cell_bytes.split_off(TOKEN_BYTES));
            let stake_delta = stake_item(&stake.delta());

            if stake_delta.inauguration_epoch < self.current_epoch + INAUGURATION {
                continue;
            }

            cells.insert(staker.clone(), cell);
            total_stake_amounts.insert(staker.clone(), total_stake_amount);

            let smt_staker = SmtStaker::from(staker.0);
            if new_smt.contains_key(&smt_staker) {
                let origin_stake_amount = new_smt.get(&smt_staker).unwrap().to_owned();
                if stake_delta.is_increase {
                    new_smt.insert(smt_staker, origin_stake_amount + stake_delta.amount);
                } else {
                    let withdraw_amount = if origin_stake_amount < stake_delta.amount {
                        origin_stake_amount
                    } else {
                        stake_delta.amount
                    };
                    new_smt.insert(smt_staker, origin_stake_amount - withdraw_amount);
                    withdraw_amounts.insert(staker, withdraw_amount);
                }
            } else {
                if !stake_delta.is_increase {
                    return Err(CkbTxErr::Increase(stake_delta.is_increase).into());
                }
                new_smt.insert(smt_staker, stake_delta.amount);
            }
        }

        let non_top_stakers = self.remove_non_top_stakers(&old_smt, &mut new_smt);

        for (staker, in_smt) in non_top_stakers.iter() {
            let smt_staker = SmtStaker::from(staker.0);
            if *in_smt {
                withdraw_amounts
                    .insert(staker.clone(), old_smt.get(&smt_staker).unwrap().to_owned());
                if !total_stake_amounts.contains_key(staker) {
                    let stake_lock = stake_lock(
                        &self.ckb.network_type,
                        &self.type_ids.metadata_type_id,
                        staker,
                    );

                    let cell = get_stake_cell(&self.ckb.client, stake_lock, xudt.clone())
                        .await?
                        .unwrap();
                    let cell_bytes = cell.output_data.clone().unwrap().into_bytes();
                    let total_stake_amount = new_u128(&cell_bytes[..TOKEN_BYTES]);

                    cells.insert(staker.clone(), cell);

                    total_stake_amounts.insert(staker.clone(), total_stake_amount);
                }
            } else {
                total_stake_amounts.remove(staker);
            }
        }

        let new_smt_stakers = new_smt
            .iter()
            .map(|(k, v)| UserAmount {
                user:        k.to_owned(),
                amount:      v.to_owned(),
                is_increase: true,
            })
            .collect();

        self.stake_smt_storage
            .insert(self.current_epoch + 2, new_smt_stakers)
            .await?;

        let new_root = self
            .stake_smt_storage
            .get_sub_root(self.current_epoch + 2)
            .await?
            .unwrap();

        Ok((
            stake_smt_cell_data(&new_root).as_bytes(),
            cells,
            Statistics {
                non_top_stakers,
                withdraw_amounts,
                total_stake_amounts,
            },
        ))
    }

    fn remove_non_top_stakers(
        &self,
        old_smt: &HashMap<SmtStaker, Amount>,
        new_smt: &mut HashMap<SmtStaker, Amount>,
    ) -> NonTopStakers {
        if new_smt.len() <= 3 * self.quorum as usize {
            return HashMap::default();
        }

        let mut all_stakes = new_smt
            .clone()
            .into_iter()
            .map(|(k, v)| (k, v))
            .collect::<Vec<(SmtStaker, Amount)>>();
        all_stakes.sort_unstable_by_key(|v| v.1);

        let delete_count = all_stakes.len() - 3 * self.quorum as usize;
        let non_top_stakers = &all_stakes[..delete_count];

        non_top_stakers
            .iter()
            .map(|(staker, _)| {
                new_smt.remove(staker);
                (TxStaker::from(staker.0), old_smt.contains_key(staker))
            })
            .collect()
    }
}
