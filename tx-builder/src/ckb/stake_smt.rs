use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use axon_types::{
    basic::Byte32,
    stake::{
        StakeArgs, StakeAtCellData, StakeSmtCellData, StakeSmtUpdateInfo as AStakeSmtUpdateInfo,
    },
};
use ckb_types::{
    bytes::Bytes,
    core::{Capacity, TransactionBuilder, TransactionView},
    packed::{CellInput, CellOutput},
    prelude::{Entity, Pack},
};

use common::types::tx_builder::{StakeItem, StakeSmtTypeIds, Staker as TxStaker};
use common::{
    traits::smt::StakeSmtStorage,
    types::{
        ckb_rpc_client::Cell,
        tx_builder::{Amount, CkbNetwork, Epoch, InStakeSmt, NonTopStakers, PrivateKey},
    },
};
use common::{
    traits::{ckb_rpc_client::CkbRpc, tx_builder::IStakeSmtTxBuilder},
    types::smt::Root,
};
use common::{
    types::smt::{Staker as SmtStaker, UserAmount},
    utils::convert::new_u128,
};
use molecule::prelude::Builder;

use crate::ckb::define::{
    constants::{INAUGURATION, TOKEN_BYTES},
    error::CkbTxErr,
    types::{StakeInfo, StakeSmtUpdateInfo},
};

use crate::ckb::utils::{
    cell_collector::{get_stake_cell, get_unique_cell, get_withdraw_cell},
    cell_data::{stake_item, token_cell_data, update_withdraw_data},
    cell_dep::{
        checkpoint_cell_dep, metadata_cell_dep, omni_lock_dep, secp256k1_lock_dep, stake_dep,
        withdraw_lock_dep, xudt_type_dep,
    },
    omni::{omni_eth_address, omni_eth_witness_placeholder},
    script::{always_success_lock, omni_eth_lock, stake_lock, stake_smt_type, xudt_type},
    tx::balance_tx,
};

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
        let stake_type = stake_smt_type(&self.ckb.network_type, &self.type_ids.stake_smt_type_id);

        let stake_smt_cell = get_unique_cell(&self.ckb.client, stake_type.clone()).await?;

        let mut inputs = vec![
            // stake smt cell
            CellInput::new_builder()
                .previous_output(stake_smt_cell.out_point.clone().into())
                .build(),
        ];

        let (root, cells, statistics) = self.collect().await?;

        let old_stake_smt_cell_bytes = stake_smt_cell.output_data.unwrap().into_bytes();
        let old_stake_smt_cell_data = StakeSmtCellData::new_unchecked(old_stake_smt_cell_bytes);
        let new_stake_smt_cell_data = old_stake_smt_cell_data
            .as_builder()
            .smt_root(Byte32::from_slice(root.as_slice()).unwrap())
            .build()
            .as_bytes();

        let mut outputs = vec![
            // stake smt cell
            CellOutput::new_builder()
                .lock(stake_lock.clone())
                .type_(Some(stake_type.clone()).pack())
                .build_exact_capacity(Capacity::bytes(new_stake_smt_cell_data.len())?)?,
        ];

        let mut outputs_data = vec![new_stake_smt_cell_data];

        // insert stake AT cells and withdraw AT cells to outputs
        self.fill_tx(
            &statistics,
            &cells,
            &mut inputs,
            &mut outputs,
            &mut outputs_data,
        )
        .await?;

        let mut cell_deps = vec![
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

        let mut witnesses = vec![
            omni_eth_witness_placeholder().as_bytes(), // Stake smt cell lock
            omni_eth_witness_placeholder().as_bytes(), // Stake AT cell lock, may not be needed
            omni_eth_witness_placeholder().as_bytes(), // capacity provider lock
        ];

        if !statistics.withdraw_amounts.is_empty() {
            cell_deps.push(withdraw_lock_dep(&self.ckb.network_type));
            witnesses.push(omni_eth_witness_placeholder().as_bytes()); // Withdraw AT cell lock
        }

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

            let mut old_stake_at_cell_data_bytes =
                cells[staker].output_data.clone().unwrap().into_bytes();
            let old_stake_at_cell_data =
                StakeAtCellData::new_unchecked(old_stake_at_cell_data_bytes.split_off(TOKEN_BYTES));
            let new_stake_at_cell_data = old_stake_at_cell_data
                .as_builder()
                .delta(
                    (&StakeItem {
                        is_increase:        false,
                        amount:             0,
                        inauguration_epoch: 0,
                    })
                        .into(),
                )
                .build()
                .as_bytes();

            let (stake_data, withdraw_data) = if statistics.withdraw_amounts.contains_key(staker) {
                let withdraw_amount = statistics.withdraw_amounts.get(staker).unwrap().to_owned();
                let old_withdraw_cell =
                    get_withdraw_cell(&self.ckb.client, withdraw_lock.clone(), xudt.clone())
                        .await?
                        .unwrap();

                inputs.push(
                    CellInput::new_builder()
                        .previous_output(old_withdraw_cell.out_point.clone().into())
                        .build(),
                );

                (
                    token_cell_data(total_stake_amount - withdraw_amount, new_stake_at_cell_data),
                    Some(update_withdraw_data(
                        old_withdraw_cell,
                        self.current_epoch + INAUGURATION,
                        withdraw_amount,
                    )),
                )
            } else {
                (
                    token_cell_data(total_stake_amount.to_owned(), new_stake_at_cell_data),
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

    fn get_stake_at_cell_data(&self, cell: &Cell) -> (Amount, StakeAtCellData) {
        let mut cell_data_bytes = cell.output_data.clone().unwrap().into_bytes();
        let total_stake_amount = new_u128(&cell_data_bytes[..TOKEN_BYTES]);
        let stake_at_cell_data =
            StakeAtCellData::new_unchecked(cell_data_bytes.split_off(TOKEN_BYTES));
        (total_stake_amount, stake_at_cell_data)
    }

    async fn update_stake_smt(&self, new_smt: HashMap<SmtStaker, Amount>) -> Result<Root> {
        let new_smt_stakers = new_smt
            .iter()
            .map(|(k, v)| UserAmount {
                user:        k.to_owned(),
                amount:      v.to_owned(),
                is_increase: true,
            })
            .collect();

        self.stake_smt_storage
            .insert(self.current_epoch + INAUGURATION, new_smt_stakers)
            .await?;

        self.stake_smt_storage.get_top_root().await
    }

    async fn collect(&self) -> Result<(Root, HashMap<TxStaker, Cell>, Statistics)> {
        let old_smt = self
            .stake_smt_storage
            .get_sub_leaves(self.current_epoch + INAUGURATION)
            .await?;

        let xudt = xudt_type(&self.ckb.network_type, &self.type_ids.xudt_owner.pack());

        let mut new_smt = old_smt.clone();
        let mut withdraw_amounts = HashMap::new(); // records all the stakers' withdraw amounts
        let mut total_stake_amounts = HashMap::new(); // records all the stakers' total stake amounts which require to be updated
        let mut cells = HashMap::new();

        for cell in self.stake_cells.clone().into_iter() {
            let staker = TxStaker::from_slice(
                &StakeArgs::new_unchecked(cell.output.lock.args.as_bytes().to_owned().into())
                    .stake_addr()
                    .as_bytes(),
            )
            .unwrap();

            let (total_stake_amount, stake) = self.get_stake_at_cell_data(&cell);
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
                } else if origin_stake_amount < stake_delta.amount {
                    withdraw_amounts.insert(staker, origin_stake_amount);
                } else {
                    new_smt.insert(smt_staker, origin_stake_amount - stake_delta.amount);
                    withdraw_amounts.insert(staker, stake_delta.amount);
                };
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

                // It represents the case where the staker doesn't update its staking but is
                // removed from the smt since it's no longer the top stakers. In this case, the
                // staker's stake at cell needs to be updated. So the cell should be put to the
                // inputs.
                if !total_stake_amounts.contains_key(staker) {
                    let cell = get_stake_cell(
                        &self.ckb.client,
                        stake_lock(
                            &self.ckb.network_type,
                            &self.type_ids.metadata_type_id,
                            staker,
                        ),
                        xudt.clone(),
                    )
                    .await?
                    .unwrap();

                    let (total_stake_amount, _) = self.get_stake_at_cell_data(&cell);

                    cells.insert(staker.clone(), cell);
                    total_stake_amounts.insert(staker.clone(), total_stake_amount);
                }
            } else {
                total_stake_amounts.remove(staker);
                cells.remove(staker);
            }
        }

        // get the old epoch proof for witness
        let old_epoch_proof = self
            .stake_smt_storage
            .generate_sub_proof(
                self.current_epoch + INAUGURATION,
                old_smt.clone().into_keys().collect(),
            )
            .await?;

        let new_root = self.update_stake_smt(new_smt.clone()).await?;

        // get the new epoch proof for witness
        let new_epoch_proof = self
            .stake_smt_storage
            .generate_sub_proof(
                self.current_epoch + INAUGURATION,
                new_smt.into_keys().collect(),
            )
            .await?;

        let _stake_smt_witness = AStakeSmtUpdateInfo::from(StakeSmtUpdateInfo {
            all_stake_infos: old_smt
                .iter()
                .map(|(k, v)| StakeInfo {
                    addr:   k.0.into(),
                    amount: v.to_owned(),
                })
                .collect(),
            old_epoch_proof,
            new_epoch_proof,
        });

        Ok((new_root, cells, Statistics {
            non_top_stakers,
            withdraw_amounts,
            total_stake_amounts,
        }))
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
