use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use axon_types::delegate::{
    DelegateArgs, DelegateAtCellData, DelegateAtCellLockData as ADelegateAtCellLockData,
    DelegateCellData, DelegateInfoDeltas, DelegateSmtCellData as ADelegateSmtCellData,
};
use ckb_types::{
    bytes::Bytes,
    core::{Capacity, TransactionBuilder, TransactionView},
    packed::{CellInput, CellOutput},
    prelude::{Entity, Pack},
};

use common::traits::{
    ckb_rpc_client::CkbRpc, smt::DelegateSmtStorage, tx_builder::IDelegateSmtTxBuilder,
};
use common::types::smt::{Delegator as SmtDelegator, Staker as SmtStaker};
use common::types::tx_builder::{
    Amount, CkbNetwork, DelegateItem, DelegateSmtTypeIds, Delegator, Epoch, InDelegateSmt,
    InStakeSmt, NonTopDelegators, PrivateKey, Staker as TxStaker,
};
use common::types::{ckb_rpc_client::Cell, smt::UserAmount};
use common::utils::convert::{new_u128, to_uint128, to_usize};
use molecule::prelude::Builder;

use crate::ckb::define::{
    constants::{INAUGURATION, TOKEN_BYTES},
    error::CkbTxErr,
    types::{DelegateAtCellLockData, DelegateSmtCellData, StakerSmtRoot},
};
use crate::ckb::utils::cell_data::{delegate_item, token_cell_data, update_withdraw_data};
use crate::ckb::utils::{
    cell_collector::{
        get_delegate_cell, get_delegate_requirement_cell, get_unique_cell, get_withdraw_cell,
    },
    cell_dep::{
        checkpoint_cell_dep, delegate_lock_dep, delegate_smt_type_dep, metadata_cell_dep,
        omni_lock_dep, secp256k1_lock_dep, withdraw_lock_dep, xudt_type_dep,
    },
    omni::{omni_eth_address, omni_eth_witness_placeholder},
    script::{
        always_success_lock, delegate_lock, delegate_requirement_type, delegate_smt_type,
        omni_eth_lock, withdraw_lock, xudt_type,
    },
    tx::balance_tx,
};

pub struct DelegateSmtTxBuilder<C: CkbRpc, D: DelegateSmtStorage> {
    ckb:                   CkbNetwork<C>,
    kicker:                PrivateKey,
    current_epoch:         Epoch,
    type_ids:              DelegateSmtTypeIds,
    delegate_cells:        Vec<Cell>,
    delegate_smt_storage:  D,
    inputs_delegate_cells: HashMap<Delegator, Cell>,
}

#[async_trait]
impl<C: CkbRpc, D: DelegateSmtStorage> IDelegateSmtTxBuilder<C, D> for DelegateSmtTxBuilder<C, D> {
    fn new(
        ckb: CkbNetwork<C>,
        kicker: PrivateKey,
        current_epoch: Epoch,
        type_ids: DelegateSmtTypeIds,
        delegate_cells: Vec<Cell>,
        delegate_smt_storage: D,
    ) -> Self {
        Self {
            ckb,
            kicker,
            current_epoch,
            type_ids,
            delegate_cells,
            delegate_smt_storage,
            inputs_delegate_cells: HashMap::new(),
        }
    }

    async fn build_tx(&mut self) -> Result<(TransactionView, NonTopDelegators)> {
        let delegate_smt_type =
            delegate_smt_type(&self.ckb.network_type, &self.type_ids.delegate_smt_type_id);

        let delegate_smt_cell =
            get_unique_cell(&self.ckb.client, delegate_smt_type.clone()).await?;

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
                .lock(always_success_lock(&self.ckb.network_type))
                .type_(Some(delegate_smt_type).pack())
                .build_exact_capacity(Capacity::bytes(root.len())?)?,
        ];

        let mut outputs_data = vec![root];

        // insert delegate AT cells and withdraw AT cells to the tx
        self.fill_tx(&statistics, &mut inputs, &mut outputs, &mut outputs_data)
            .await?;

        let cell_deps = vec![
            secp256k1_lock_dep(&self.ckb.network_type),
            omni_lock_dep(&self.ckb.network_type),
            xudt_type_dep(&self.ckb.network_type),
            delegate_lock_dep(&self.ckb.network_type),
            delegate_smt_type_dep(&self.ckb.network_type),
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

        // todo
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
    pub withdraw_amounts:   HashMap<Delegator, HashMap<TxStaker, Amount>>,
    pub non_top_delegators: HashMap<Delegator, HashMap<TxStaker, InDelegateSmt>>,
}

impl<C: CkbRpc, D: DelegateSmtStorage> DelegateSmtTxBuilder<C, D> {
    async fn fill_tx(
        &self,
        statistics: &Statistics,
        inputs: &mut Vec<CellInput>,
        outputs: &mut Vec<CellOutput>,
        outputs_data: &mut Vec<Bytes>,
    ) -> Result<()> {
        let xudt = xudt_type(&self.ckb.network_type, &self.type_ids.xudt_owner.pack());

        for (delegator, delegate_cell) in self.inputs_delegate_cells.iter() {
            // inputs: delegate AT cell
            inputs.push(
                CellInput::new_builder()
                    .previous_output(delegate_cell.out_point.clone().into())
                    .build(),
            );

            let (old_total_delegate_amount, old_delegate_data) =
                self.parse_delegate_data(delegate_cell);

            let withdraw_lock = withdraw_lock(
                &self.ckb.network_type,
                &self.type_ids.metadata_type_id,
                delegator,
            );

            let (new_delegate_data, new_withdraw_data) = if statistics
                .withdraw_amounts
                .contains_key(delegator)
            {
                let old_withdraw_cell =
                    get_withdraw_cell(&self.ckb.client, withdraw_lock.clone(), xudt.clone())
                        .await?
                        .unwrap();

                // inputs: withdraw AT cell
                inputs.push(
                    CellInput::new_builder()
                        .previous_output(old_withdraw_cell.out_point.clone().into())
                        .build(),
                );

                let withdraw_amounts = statistics.withdraw_amounts.get(delegator).unwrap();

                let total_withdraw_amount = withdraw_amounts
                    .values()
                    .fold(0_u128, |acc, x| acc + x.to_owned());
                let mut delegator_infos: Vec<DelegateItem> = Vec::new();

                for delegate in old_delegate_data.lock().delegator_infos() {
                    let mut delegate_item = delegate_item(&delegate);

                    delegator_infos.push(if withdraw_amounts.contains_key(&delegate_item.staker) {
                        let withdraw_amount = withdraw_amounts
                            .get(&delegate_item.staker)
                            .unwrap()
                            .to_owned();
                        if delegate_item.total_amount < withdraw_amount {
                            return Err(CkbTxErr::ExceedTotalAmount {
                                total_amount: delegate_item.total_amount,
                                new_amount:   withdraw_amount,
                            }
                            .into());
                        }
                        delegate_item.total_amount -= withdraw_amount;
                        delegate_item
                    } else {
                        delegate_item
                    });
                }

                let new_delegate_data = old_delegate_data
                    .as_builder()
                    .lock(ADelegateAtCellLockData::from(DelegateAtCellLockData {
                        delegator_infos,
                    }))
                    .build()
                    .as_bytes();

                (
                    token_cell_data(
                        old_total_delegate_amount - total_withdraw_amount,
                        new_delegate_data,
                    ),
                    Some(update_withdraw_data(
                        old_withdraw_cell,
                        self.current_epoch + INAUGURATION,
                        total_withdraw_amount,
                    )),
                )
            } else {
                let mut new_delegates = DelegateInfoDeltas::new_builder();

                for delegate in old_delegate_data.lock().delegator_infos() {
                    new_delegates =
                        new_delegates.push(delegate.as_builder().amount(to_uint128(0)).build());
                }

                let new_delegate_data = old_delegate_data.lock().clone();
                let new_delegate_data = old_delegate_data
                    .as_builder()
                    .lock(
                        new_delegate_data
                            .as_builder()
                            .delegator_infos(new_delegates.build())
                            .build(),
                    )
                    .build()
                    .as_bytes();

                (
                    token_cell_data(old_total_delegate_amount.to_owned(), new_delegate_data),
                    None,
                )
            };

            // outputs: delegate AT cell
            outputs.push(
                CellOutput::new_builder()
                    .lock(delegate_lock(
                        &self.ckb.network_type,
                        &self.type_ids.metadata_type_id,
                        delegator,
                    ))
                    .type_(Some(xudt.clone()).pack())
                    .build_exact_capacity(Capacity::bytes(new_delegate_data.len())?)?,
            );
            outputs_data.push(new_delegate_data);

            // outputs: withdraw AT cell
            if new_withdraw_data.is_some() {
                outputs.push(
                    CellOutput::new_builder()
                        .lock(withdraw_lock)
                        .type_(Some(xudt.clone()).pack())
                        .build_exact_capacity(Capacity::bytes(
                            new_withdraw_data.as_ref().unwrap().len(),
                        )?)?,
                );
                outputs_data.push(new_withdraw_data.unwrap());
            }
        }

        Ok(())
    }

    fn parse_delegate_data(&self, cell: &Cell) -> (Amount, DelegateAtCellData) {
        let mut cell_data_bytes = cell.output_data.clone().unwrap().into_bytes();
        let total_delegate_amount = new_u128(&cell_data_bytes[..TOKEN_BYTES]);
        let delegate_data =
            DelegateAtCellData::new_unchecked(cell_data_bytes.split_off(TOKEN_BYTES));
        (total_delegate_amount, delegate_data)
    }

    async fn collect(&mut self) -> Result<(Bytes, Statistics)> {
        let mut delegates = HashMap::new();
        self.collect_cell_delegates(&mut delegates)?;

        let mut non_top_delegators = HashMap::new();
        let mut withdraw_amounts = HashMap::new();
        let mut new_roots = vec![];

        for (staker, delegators) in delegates.iter() {
            let old_smt = self
                .delegate_smt_storage
                .get_sub_leaves(
                    self.current_epoch + INAUGURATION,
                    SmtStaker::from_slice(staker.as_bytes()),
                )
                .await?;

            let mut new_smt = old_smt.clone();

            for (delegator, delegate) in delegators.iter() {
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
                        withdraw_amounts
                            .entry(delegator.clone())
                            .and_modify(|e: &mut HashMap<TxStaker, u128>| {
                                e.insert(staker.clone(), withdraw_amount);
                            })
                            .or_insert_with(HashMap::new)
                            .insert(staker.clone(), withdraw_amount);
                        new_smt.insert(smt_delegator, origin_amount - withdraw_amount);
                    }
                } else {
                    if !delegate.is_increase {
                        return Err(CkbTxErr::Increase(delegate.is_increase).into());
                    }
                    new_smt.insert(smt_delegator, delegate.amount);
                }
            }

            self.collect_non_top_delegators(
                staker.clone(),
                &old_smt,
                &mut new_smt,
                &mut withdraw_amounts,
                &mut non_top_delegators,
            )
            .await?;

            new_roots.push(self.update_delegate_smt(staker.clone(), new_smt).await?);
        }

        Ok((
            ADelegateSmtCellData::from(DelegateSmtCellData {
                metadata_type_id: self.type_ids.metadata_type_id.clone(),
                smt_roots:        new_roots,
            })
            .as_bytes(),
            Statistics {
                non_top_delegators,
                withdraw_amounts,
            },
        ))
    }

    fn collect_cell_delegates(
        &mut self,
        delegates: &mut HashMap<TxStaker, HashMap<Delegator, DelegateItem>>,
    ) -> Result<()> {
        for cell in self.delegate_cells.clone().into_iter() {
            let delegator = Delegator::from_slice(
                &DelegateArgs::new_unchecked(cell.output.lock.args.as_bytes().to_owned().into())
                    .delegator_addr()
                    .as_bytes(),
            )?;

            let mut cell_bytes = cell.output_data.clone().unwrap().into_bytes();

            let delegate = &DelegateAtCellData::new_unchecked(cell_bytes.split_off(TOKEN_BYTES));
            let delegate_infos = delegate.lock().delegator_infos();
            let mut expired = false;

            for info in delegate_infos.into_iter() {
                let item = delegate_item(&info);
                if item.inauguration_epoch < self.current_epoch + INAUGURATION {
                    expired = true;
                    break;
                } else {
                    delegates
                        .entry(item.staker.clone())
                        .and_modify(|e: &mut HashMap<ckb_types::H160, DelegateItem>| {
                            e.insert(delegator.clone(), item.clone());
                        })
                        .or_insert_with(HashMap::new)
                        .insert(delegator.clone(), item.clone());
                }
            }

            if !expired {
                self.inputs_delegate_cells.insert(delegator.clone(), cell);
            }
        }

        Ok(())
    }

    async fn collect_non_top_delegators(
        &mut self,
        staker: TxStaker,
        old_smt: &HashMap<SmtDelegator, Amount>,
        new_smt: &mut HashMap<SmtDelegator, Amount>,
        withdraw_amounts: &mut HashMap<Delegator, HashMap<TxStaker, Amount>>,
        non_top_delegators: &mut HashMap<Delegator, HashMap<TxStaker, InStakeSmt>>,
    ) -> Result<()> {
        let maximum_delegators = self.get_maximum_delegators(&staker).await?;

        if new_smt.len() <= maximum_delegators {
            return Ok(());
        }

        let mut all_delegates = new_smt
            .clone()
            .into_iter()
            .map(|(k, v)| (k, v))
            .collect::<Vec<(SmtDelegator, Amount)>>();

        all_delegates.sort_unstable_by_key(|v| v.1);

        let delete_count = all_delegates.len() - maximum_delegators;
        let deleted_delegators = &all_delegates[..delete_count];
        let xudt = xudt_type(&self.ckb.network_type, &self.type_ids.xudt_owner.pack());

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

                if !self.inputs_delegate_cells.contains_key(&tx_delegator) {
                    let cell = get_delegate_cell(
                        &self.ckb.client,
                        delegate_lock(
                            &self.ckb.network_type,
                            &self.type_ids.metadata_type_id,
                            &tx_delegator,
                        ),
                        xudt.clone(),
                    )
                    .await?
                    .unwrap();

                    self.inputs_delegate_cells.insert(staker.clone(), cell);
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

    async fn update_delegate_smt(
        &self,
        staker: TxStaker,
        new_smt: HashMap<SmtDelegator, Amount>,
    ) -> Result<StakerSmtRoot> {
        let new_delegators = new_smt
            .into_iter()
            .map(|(k, v)| UserAmount {
                user:        k,
                amount:      v,
                is_increase: true,
            })
            .collect();

        let smt_staker = SmtStaker::from_slice(staker.as_bytes());

        self.delegate_smt_storage
            .insert(
                self.current_epoch + INAUGURATION,
                smt_staker,
                new_delegators,
            )
            .await?;

        Ok(StakerSmtRoot {
            staker,
            root: self.delegate_smt_storage.get_top_root(smt_staker).await?,
        })
    }

    async fn get_maximum_delegators(&self, staker: &TxStaker) -> Result<usize> {
        let delegate_requirement_cell = get_delegate_requirement_cell(
            &self.ckb.client,
            omni_eth_lock(&self.ckb.network_type, staker),
            delegate_requirement_type(
                &self.ckb.network_type,
                &self.type_ids.metadata_type_id,
                staker,
            ),
        )
        .await?;

        if delegate_requirement_cell.is_none() {
            return Err(CkbTxErr::CellNotFound("DelegateRequirement".to_owned()).into());
        }

        let delegate_requirement_cell_bytes = delegate_requirement_cell
            .unwrap()
            .output_data
            .unwrap()
            .into_bytes();
        let delegate_cell_info = DelegateCellData::new_unchecked(delegate_requirement_cell_bytes);
        let maximum_delegators = to_usize(
            delegate_cell_info
                .delegate_requirement()
                .max_delegator_size(),
        );
        Ok(maximum_delegators)
    }
}
