use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use ckb_types::{
    bytes::Bytes,
    core::{Capacity, TransactionBuilder, TransactionView},
    packed::{CellInput, CellOutput, WitnessArgs},
    prelude::{Entity, Pack},
};
use molecule::prelude::Builder;

use common::traits::{
    ckb_rpc_client::CkbRpc, smt::DelegateSmtStorage, tx_builder::IDelegateSmtTxBuilder,
};
use common::types::axon_types::delegate::{
    DelegateArgs, DelegateAtCellData, DelegateAtCellLockData as ADelegateAtCellLockData,
    DelegateCellData, DelegateInfoDeltas, DelegateSmtCellData as ADelegateSmtCellData,
};
use common::types::smt::Delegator as SmtDelegator;
use common::types::tx_builder::{
    Amount, DelegateItem, DelegateSmtTypeIds, Delegator, Epoch, InDelegateSmt, InStakeSmt,
    NonTopDelegators, PrivateKey, Staker as TxStaker,
};
use common::types::{ckb_rpc_client::Cell, smt::UserAmount};
use common::utils::convert::{new_u128, to_ckb_h160, to_eth_h160, to_uint128, to_usize};

use crate::ckb::define::types::{DelegateInfo, StakeGroupInfo};
use crate::ckb::define::{
    constants::{INAUGURATION, TOKEN_BYTES},
    error::CkbTxErr,
    types::{DelegateAtCellLockData, DelegateSmtCellData, StakerSmtRoot},
};
use crate::ckb::helper::{
    token_cell_data, AlwaysSuccess, Checkpoint, Delegate, Metadata, OmniEth, Secp256k1, Tx,
    Withdraw, Xudt,
};

pub struct DelegateSmtTxBuilder<'a, C: CkbRpc, D: DelegateSmtStorage> {
    ckb:                   &'a C,
    kicker:                PrivateKey,
    current_epoch:         Epoch,
    type_ids:              DelegateSmtTypeIds,
    delegate_cells:        Vec<Cell>,
    delegate_smt_storage:  D,
    inputs_delegate_cells: HashMap<Delegator, Cell>,
}

#[async_trait]
impl<'a, C: CkbRpc, D: DelegateSmtStorage> IDelegateSmtTxBuilder<'a, C, D>
    for DelegateSmtTxBuilder<'a, C, D>
{
    fn new(
        ckb: &'a C,
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
        let delegate_smt_type = Delegate::smt_type(&self.type_ids.delegate_smt_type_id);

        let delegate_smt_cell = Delegate::get_smt_cell(self.ckb, delegate_smt_type.clone()).await?;

        let mut inputs = vec![
            // delegate smt cell
            CellInput::new_builder()
                .previous_output(delegate_smt_cell.out_point.clone().into())
                .build(),
        ];

        let (root, statistics, smt_witness) = self.collect().await?;

        let mut outputs = vec![
            // delegate smt cell
            CellOutput::new_builder()
                .lock(AlwaysSuccess::lock())
                .type_(Some(delegate_smt_type).pack())
                .build_exact_capacity(Capacity::bytes(root.len())?)?,
        ];

        let mut outputs_data = vec![root];

        // insert delegate AT cells and withdraw AT cells to the tx
        self.fill_tx(&statistics, &mut inputs, &mut outputs, &mut outputs_data)
            .await?;

        let cell_deps = vec![
            Secp256k1::lock_dep(),
            OmniEth::lock_dep(),
            AlwaysSuccess::lock_dep(),
            Xudt::type_dep(),
            Delegate::lock_dep(),
            Delegate::smt_type_dep(),
            Checkpoint::cell_dep(self.ckb, &self.type_ids.checkpoint_type_id).await?,
            Metadata::cell_dep(self.ckb, &self.type_ids.metadata_type_id).await?,
            Withdraw::lock_dep(),
        ];

        // todo
        let witnesses = vec![
            smt_witness.as_bytes(),                    // Delegate AT cell lock
            OmniEth::witness_placeholder().as_bytes(), // Withdraw AT cell lock
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

        let omni_eth = OmniEth::new(self.kicker.clone());
        let kicker_lock = OmniEth::lock(&omni_eth.address()?);
        let tx = Tx::new(self.ckb, tx).balance(kicker_lock).await?;

        // todo: sign tx

        Ok((tx, statistics.non_top_delegators))
    }
}

struct Statistics {
    pub withdraw_amounts:   HashMap<Delegator, HashMap<TxStaker, Amount>>,
    pub non_top_delegators: HashMap<Delegator, HashMap<TxStaker, InDelegateSmt>>,
}

impl<'a, C: CkbRpc, D: DelegateSmtStorage> DelegateSmtTxBuilder<'a, C, D> {
    async fn fill_tx(
        &self,
        statistics: &Statistics,
        inputs: &mut Vec<CellInput>,
        outputs: &mut Vec<CellOutput>,
        outputs_data: &mut Vec<Bytes>,
    ) -> Result<()> {
        let xudt = Xudt::type_(&self.type_ids.xudt_owner.pack());

        for (delegator, delegate_cell) in self.inputs_delegate_cells.iter() {
            // inputs: delegate AT cell
            inputs.push(
                CellInput::new_builder()
                    .previous_output(delegate_cell.out_point.clone().into())
                    .build(),
            );

            let (old_total_delegate_amount, old_delegate_data) =
                self.parse_delegate_data(delegate_cell);

            let withdraw_lock = Withdraw::lock(&self.type_ids.metadata_type_id, delegator);

            let (new_delegate_data, new_withdraw_data) = if statistics
                .withdraw_amounts
                .contains_key(delegator)
            {
                let old_withdraw_cell =
                    Withdraw::get_cell(self.ckb, withdraw_lock.clone(), xudt.clone())
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
                    let mut delegate_item = Delegate::item(&delegate);

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
                    Some(Withdraw::update_cell_data(
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

                let inner_delegate_data = old_delegate_data.lock();
                let new_delegate_data = old_delegate_data
                    .as_builder()
                    .lock(
                        inner_delegate_data
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
                    .lock(Delegate::lock(&self.type_ids.metadata_type_id, delegator))
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

    async fn collect(&mut self) -> Result<(Bytes, Statistics, WitnessArgs)> {
        let mut delegates = HashMap::new();
        self.collect_cell_delegates(&mut delegates)?;

        let mut non_top_delegators = HashMap::new();
        let mut withdraw_amounts = HashMap::new();
        let mut new_roots = vec![];
        let mut delegate_infos = vec![];

        for (staker, delegators) in delegates.iter() {
            let old_smt = self
                .delegate_smt_storage
                .get_sub_leaves(self.current_epoch + INAUGURATION, to_eth_h160(staker))
                .await?;

            // get the old epoch proof for witness
            let old_epoch_proof = self
                .delegate_smt_storage
                .generate_sub_proof(
                    to_eth_h160(staker),
                    self.current_epoch + INAUGURATION,
                    old_smt.clone().into_keys().collect(),
                )
                .await?;

            let mut new_smt = old_smt.clone();

            for (delegator, delegate) in delegators.iter() {
                let smt_delegator = to_eth_h160(delegator);
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

            // get the new epoch proof for witness
            let new_epoch_proof = self
                .delegate_smt_storage
                .generate_sub_proof(
                    to_eth_h160(staker),
                    self.current_epoch + INAUGURATION,
                    new_smt.clone().into_keys().collect(),
                )
                .await?;

            delegate_infos.push(StakeGroupInfo {
                staker:                   staker.clone(),
                delegate_old_epoch_proof: old_epoch_proof,
                delegate_new_epoch_proof: new_epoch_proof,
                delegate_infos:           new_smt
                    .clone()
                    .into_iter()
                    .map(|(addr, amount)| DelegateInfo {
                        delegator_addr: to_ckb_h160(&addr),
                        amount,
                    })
                    .collect(),
            });

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
            Delegate::smt_witness(delegate_infos),
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
                let item = Delegate::item(&info);
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
        let xudt = Xudt::type_(&self.type_ids.xudt_owner.pack());

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
                    let cell = Delegate::get_cell(
                        self.ckb,
                        Delegate::lock(&self.type_ids.metadata_type_id, &tx_delegator),
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

        let smt_staker = to_eth_h160(&staker);

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
        let delegate_requirement_cell = Delegate::get_requirement_cell(
            self.ckb,
            Delegate::requirement_type(&self.type_ids.metadata_type_id, staker),
        )
        .await?;

        let delegate_requirement_cell_bytes =
            delegate_requirement_cell.output_data.unwrap().into_bytes();
        let delegate_cell_info = DelegateCellData::new_unchecked(delegate_requirement_cell_bytes);
        let maximum_delegators = to_usize(
            delegate_cell_info
                .delegate_requirement()
                .max_delegator_size(),
        );
        Ok(maximum_delegators)
    }
}
