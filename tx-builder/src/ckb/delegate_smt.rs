use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use ckb_sdk::{ScriptGroup, ScriptGroupType};
use ckb_types::{
    bytes::Bytes,
    core::{Capacity, TransactionBuilder, TransactionView},
    packed::{CellDep, CellInput, CellOutput, WitnessArgs},
    prelude::{Entity, Pack},
    H160,
};
use molecule::prelude::Builder;

use common::traits::{
    ckb_rpc_client::CkbRpc, smt::DelegateSmtStorage, tx_builder::IDelegateSmtTxBuilder,
};
use common::types::axon_types::delegate::{
    DelegateArgs, DelegateAtCellData, DelegateAtCellLockData as ADelegateAtCellLockData,
    DelegateCellData, DelegateSmtCellData as ADelegateSmtCellData,
};
use common::types::ckb_rpc_client::Cell;
use common::types::smt::{Delegator as SmtDelegator, UserAmount};
use common::types::tx_builder::{
    Amount, DelegateItem, DelegateSmtTypeIds, Delegator, Epoch, InDelegateSmt, InStakeSmt,
    NonTopDelegators, PrivateKey, Staker,
};
use common::utils::convert::{new_u128, to_ckb_h160, to_eth_h160, to_h160, to_usize};

use crate::ckb::define::types::{DelegateInfo, StakeGroupInfo};
use crate::ckb::define::{
    constants::{INAUGURATION, TOKEN_BYTES},
    error::CkbTxErr,
    types::{DelegateAtCellLockData, DelegateSmtCellData, StakerSmtRoot},
};
use crate::ckb::helper::{
    token_cell_data, AlwaysSuccess, Checkpoint, Delegate, Metadata, OmniEth, Secp256k1, Stake, Tx,
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
    maximum_delegators:    HashMap<Staker, usize>,
    stake_cell_deps:       Vec<CellDep>,
    requirement_cell_deps: Vec<CellDep>,
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
            maximum_delegators: HashMap::new(),
            stake_cell_deps: Vec::new(),
            requirement_cell_deps: Vec::new(),
        }
    }

    async fn build_tx(mut self) -> Result<(TransactionView, NonTopDelegators)> {
        let delegate_smt_type = Delegate::smt_type(&self.type_ids.delegate_smt_type_id);
        let delegate_smt_cell = Delegate::get_smt_cell(self.ckb, delegate_smt_type.clone()).await?;

        let mut inputs = vec![
            // delegate smt cell
            CellInput::new_builder()
                .previous_output(delegate_smt_cell.out_point.clone().into())
                .build(),
        ];

        let (new_roots, statistics, smt_witness) = self.process_delegation().await?;

        let smt_data = self.generate_smt_data(self.parse_old_roots(delegate_smt_cell), new_roots);

        let mut outputs = vec![
            // delegate smt cell
            CellOutput::new_builder()
                .lock(AlwaysSuccess::lock())
                .type_(Some(delegate_smt_type).pack())
                .build_exact_capacity(Capacity::bytes(smt_data.len())?)?,
        ];
        let mut outputs_data = vec![smt_data];
        let mut witnesses = vec![smt_witness.as_bytes()];

        // insert delegate AT cells and withdraw AT cells to the tx
        self.fill_tx(
            &statistics,
            &mut inputs,
            &mut outputs,
            &mut outputs_data,
            &mut witnesses,
        )
        .await?;

        let mut cell_deps = vec![
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
        cell_deps.extend(self.stake_cell_deps);
        cell_deps.extend(self.requirement_cell_deps);

        witnesses.push(OmniEth::witness_placeholder().as_bytes()); // capacity provider lock

        let tx = TransactionBuilder::default()
            .inputs(inputs)
            .outputs(outputs)
            .outputs_data(outputs_data.pack())
            .cell_deps(cell_deps)
            .witnesses(witnesses.pack())
            .build();

        let omni_eth = OmniEth::new(self.kicker.clone());
        let kicker_lock = OmniEth::lock(&omni_eth.address()?);

        let mut tx = Tx::new(self.ckb, tx);
        tx.balance(kicker_lock.clone()).await?;

        tx.sign(&omni_eth.signer()?, &ScriptGroup {
            script:         kicker_lock,
            group_type:     ScriptGroupType::Lock,
            input_indices:  vec![tx.inner_ref().witnesses().len() - 1],
            output_indices: vec![],
        })?;

        Ok((tx.inner(), statistics.non_top_delegators))
    }
}

struct Statistics {
    pub withdraw_amounts:   HashMap<Delegator, HashMap<Staker, Amount>>,
    pub non_top_delegators: HashMap<Delegator, HashMap<Staker, InDelegateSmt>>,
}

impl<'a, C: CkbRpc, D: DelegateSmtStorage> DelegateSmtTxBuilder<'a, C, D> {
    async fn fill_tx(
        &self,
        statistics: &Statistics,
        inputs: &mut Vec<CellInput>,
        outputs: &mut Vec<CellOutput>,
        outputs_data: &mut Vec<Bytes>,
        witnesses: &mut Vec<Bytes>,
    ) -> Result<()> {
        let xudt = Xudt::type_(&self.type_ids.xudt_owner.pack());

        for (delegator, delegate_cell) in self.inputs_delegate_cells.iter() {
            // inputs: delegate AT cell
            inputs.push(
                CellInput::new_builder()
                    .previous_output(delegate_cell.out_point.clone().into())
                    .build(),
            );
            witnesses.push(Delegate::witness(1).as_bytes());

            let (old_total_delegate_amount, old_delegate_data) =
                self.parse_delegate_data(delegate_cell);

            log::info!(
                "[delegate smt] delegator: {}, old total delegate amount: {}",
                delegator.to_string(),
                old_total_delegate_amount,
            );

            let withdraw_lock = Withdraw::lock(&self.type_ids.metadata_type_id, delegator);
            let mut total_withdraw_amount = 0;

            if statistics.withdraw_amounts.contains_key(delegator) {
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
                witnesses.push(Withdraw::witness(true).as_bytes());

                let withdraw_amounts = statistics.withdraw_amounts.get(delegator).unwrap();
                total_withdraw_amount = withdraw_amounts
                    .values()
                    .fold(0_u128, |acc, x| acc + x.to_owned());

                // outputs: withdraw AT cell
                outputs_data.push(Withdraw::update_cell_data(
                    &old_withdraw_cell,
                    self.current_epoch + INAUGURATION,
                    total_withdraw_amount,
                ));
                outputs.push(
                    CellOutput::new_builder()
                        .lock(withdraw_lock)
                        .type_(Some(xudt.clone()).pack())
                        .build_exact_capacity(Capacity::bytes(
                            outputs_data.last().unwrap().len(),
                        )?)?,
                );
            }

            if old_total_delegate_amount < total_withdraw_amount {
                return Err(CkbTxErr::ExceedTotalAmount {
                    total_amount: old_total_delegate_amount,
                    new_amount:   total_withdraw_amount,
                }
                .into());
            }

            log::info!(
                "[delegate smt] delegator: {}, new total delegate amount: {}, withdraw amount: {}",
                delegator.to_string(),
                old_total_delegate_amount - total_withdraw_amount,
                total_withdraw_amount,
            );

            let new_delegate_data = {
                let delegator_addr = old_delegate_data.lock().l2_address();
                let new_delegate_data = old_delegate_data
                    .as_builder()
                    .lock(ADelegateAtCellLockData::from(DelegateAtCellLockData {
                        l2_address:      to_h160(&delegator_addr),
                        delegator_infos: vec![],
                    }))
                    .build()
                    .as_bytes();
                token_cell_data(
                    old_total_delegate_amount - total_withdraw_amount,
                    new_delegate_data,
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
        }

        Ok(())
    }

    async fn process_delegation(
        &mut self,
    ) -> Result<(
        HashMap<ckb_types::H160, StakerSmtRoot>,
        Statistics,
        WitnessArgs,
    )> {
        let mut delegates = HashMap::new();
        self.collect_cell_delegates(&mut delegates)?;

        let mut non_top_delegators = HashMap::new();
        let mut withdraw_amounts = HashMap::new();
        let mut new_roots = HashMap::new();
        let mut smt_witness = vec![];

        for (staker, delegate) in delegates.iter() {
            let old_smt = self
                .delegate_smt_storage
                .get_sub_leaves(self.current_epoch + INAUGURATION, staker.0.into())
                .await?;

            for (user, amount) in old_smt.iter() {
                log::info!(
                    "[delegate smt] old smt, staker: {}, delegator: {}, amount: {}",
                    staker.to_string(),
                    user.to_string(),
                    amount
                );
            }

            let mut new_smt = old_smt.clone();

            self.update_amount(staker, delegate, &mut new_smt, &mut withdraw_amounts)?;

            self.collect_non_top_delegators(
                staker.clone(),
                &old_smt,
                &mut new_smt,
                &mut withdraw_amounts,
                &mut non_top_delegators,
            )
            .await?;

            for (user, amount) in new_smt.iter() {
                log::info!(
                    "[delegate smt] new smt, staker: {}, delegator: {}, amount: {}",
                    staker.to_string(),
                    user.to_string(),
                    amount
                );
            }

            // get the old epoch proof for witness
            let old_epoch_proof = self
                .delegate_smt_storage
                .generate_top_proof(vec![self.current_epoch + INAUGURATION], staker.0.into())
                .await?;

            let new_root = self.update_delegate_smt(staker.clone(), new_smt).await?;
            new_roots.insert(staker.clone(), new_root);

            // get the new epoch proof for witness
            let new_epoch_proof = self
                .delegate_smt_storage
                .generate_top_proof(vec![self.current_epoch + INAUGURATION], staker.0.into())
                .await?;

            smt_witness.push(StakeGroupInfo {
                staker:                   staker.clone(),
                delegate_old_epoch_proof: old_epoch_proof,
                delegate_new_epoch_proof: new_epoch_proof,
                delegate_infos:           old_smt
                    .into_iter()
                    .map(|(addr, amount)| DelegateInfo {
                        delegator_addr: to_ckb_h160(&addr),
                        amount,
                    })
                    .collect(),
            });
        }

        Ok((
            new_roots,
            Statistics {
                non_top_delegators,
                withdraw_amounts,
            },
            Delegate::smt_witness(0, smt_witness),
        ))
    }

    fn collect_cell_delegates(
        &mut self,
        delegates: &mut HashMap<Staker, HashMap<Delegator, DelegateItem>>,
    ) -> Result<()> {
        for cell in self.delegate_cells.clone().into_iter() {
            let delegator = Delegator::from_slice(
                &DelegateArgs::new_unchecked(cell.output.lock.args.as_bytes().to_owned().into())
                    .delegator_addr()
                    .as_bytes(),
            )?;

            let mut cell_bytes = cell.output_data.clone().unwrap().into_bytes();
            let delegate = DelegateAtCellData::new_unchecked(cell_bytes.split_off(TOKEN_BYTES));
            let delegate_infos = delegate.lock().delegator_infos();
            let mut expired = false;

            for info in delegate_infos.into_iter() {
                let item = DelegateItem::from(info);
                if item.inauguration_epoch < self.current_epoch + INAUGURATION {
                    expired = true;
                    break;
                } else {
                    log::info!(
                        "[delegate smt] delegator: {}, delta: {:?}",
                        delegator.to_string(),
                        item
                    );
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

    fn update_amount(
        &mut self,
        staker: &Staker,
        delegation: &HashMap<H160, DelegateItem>,
        new_smt: &mut HashMap<SmtDelegator, u128>,
        withdraw_amounts: &mut HashMap<Delegator, HashMap<Staker, Amount>>,
    ) -> Result<()> {
        for (delegator, delegate) in delegation.iter() {
            let smt_delegator = to_eth_h160(delegator);
            // The delegation has taken effect
            if new_smt.contains_key(&smt_delegator) {
                let smt_amount = new_smt.get(&smt_delegator).unwrap().to_owned();
                if delegate.is_increase {
                    // add delegation
                    new_smt.insert(smt_delegator, smt_amount + delegate.amount);
                } else {
                    // redeem delegation
                    let withdraw_amount = if smt_amount < delegate.amount {
                        smt_amount
                    } else {
                        delegate.amount
                    };
                    withdraw_amounts
                        .entry(delegator.clone())
                        .and_modify(|e: &mut HashMap<Staker, u128>| {
                            e.insert(staker.clone(), withdraw_amount);
                        })
                        .or_insert_with(HashMap::new)
                        .insert(staker.clone(), withdraw_amount);
                    new_smt.insert(smt_delegator, smt_amount - withdraw_amount);
                }
            } else {
                // The first delegation must be an increase in amount.
                if !delegate.is_increase {
                    return Err(CkbTxErr::Increase(delegate.is_increase).into());
                }
                new_smt.insert(smt_delegator, delegate.amount);
            }
        }
        Ok(())
    }

    async fn collect_non_top_delegators(
        &mut self,
        staker: Staker,
        old_smt: &HashMap<SmtDelegator, Amount>,
        new_smt: &mut HashMap<SmtDelegator, Amount>,
        withdraw_amounts: &mut HashMap<Delegator, HashMap<Staker, Amount>>,
        non_top_delegators: &mut HashMap<Delegator, HashMap<Staker, InStakeSmt>>,
    ) -> Result<()> {
        let maximum_delegators = self.get_maximum_delegators(&staker).await?;

        log::info!(
            "[delegate smt] staker: {}, maximum delegators: {}, delegators count: {}",
            staker.to_string(),
            maximum_delegators,
            new_smt.len(),
        );

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
            log::info!(
                "[delegate smt] none top delegator: {}, smt amount: {}",
                delegator.to_string(),
                amount
            );

            new_smt.remove(delegator);

            let tx_delegator = Delegator::from_slice(delegator.as_bytes()).unwrap();
            let mut in_smt = false;

            // Refund all delegators' money.
            if old_smt.contains_key(delegator) {
                log::info!("[delegate smt] in delegate smt");

                in_smt = true;
                let smt_amount = *old_smt.get(delegator).unwrap();

                withdraw_amounts
                    .entry(tx_delegator.clone())
                    .and_modify(|e| {
                        e.insert(staker.clone(), *amount);
                    })
                    .or_insert_with(HashMap::new)
                    .insert(staker.clone(), smt_amount);

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
        staker: Staker,
        new_smt: HashMap<SmtDelegator, Amount>,
    ) -> Result<StakerSmtRoot> {
        let new_delegators: Vec<UserAmount> = new_smt
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
                staker.0.into(),
                new_delegators,
            )
            .await?;

        Ok(StakerSmtRoot {
            staker: staker.0.into(),
            root:   self
                .delegate_smt_storage
                .get_top_root(staker.0.into())
                .await
                .unwrap(),
        })
    }

    async fn get_maximum_delegators(&mut self, staker: &Staker) -> Result<usize> {
        if self.maximum_delegators.contains_key(staker) {
            return Ok(*self.maximum_delegators.get(staker).unwrap());
        }

        let (requirement_type_id, stake_cell_outpoint) = Stake::get_delegate_requirement_type_id(
            self.ckb,
            &self.type_ids.metadata_type_id,
            staker,
            &self.type_ids.xudt_owner,
        )
        .await?;

        self.stake_cell_deps.push(
            CellDep::new_builder()
                .out_point(stake_cell_outpoint.into())
                .build(),
        );

        let delegate_requirement_cell = Delegate::get_requirement_cell(
            self.ckb,
            Delegate::requirement_type(&self.type_ids.metadata_type_id, &requirement_type_id),
        )
        .await?;

        self.requirement_cell_deps.push(
            CellDep::new_builder()
                .out_point(delegate_requirement_cell.out_point.into())
                .build(),
        );

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

    fn parse_delegate_data(&self, delegate_cell: &Cell) -> (Amount, DelegateAtCellData) {
        let mut cell_data_bytes = delegate_cell.output_data.clone().unwrap().into_bytes();
        let total_delegate_amount = new_u128(&cell_data_bytes[..TOKEN_BYTES]);
        let delegate_data =
            DelegateAtCellData::new_unchecked(cell_data_bytes.split_off(TOKEN_BYTES));
        (total_delegate_amount, delegate_data)
    }

    fn parse_old_roots(&self, delegate_smt_cell: Cell) -> HashMap<ckb_types::H160, StakerSmtRoot> {
        let smt_bytes = delegate_smt_cell.output_data.unwrap().into_bytes();
        let delegate_smt_data = ADelegateSmtCellData::new_unchecked(smt_bytes);
        let old_smt_roots = delegate_smt_data.smt_roots();

        let mut old_roots = HashMap::new();

        for old_root in old_smt_roots.into_iter() {
            let root = StakerSmtRoot::from(old_root);
            old_roots.insert(root.staker.clone(), root);
        }

        old_roots
    }

    fn generate_smt_data(
        &self,
        old_roots: HashMap<ckb_types::H160, StakerSmtRoot>,
        new_roots: HashMap<ckb_types::H160, StakerSmtRoot>,
    ) -> bytes::Bytes {
        let mut new_smt_roots = old_roots;

        // update roots
        for (staker, new_root) in new_roots.into_iter() {
            new_smt_roots.insert(staker, new_root);
        }

        ADelegateSmtCellData::from(DelegateSmtCellData {
            metadata_type_hash: Metadata::type_(&self.type_ids.metadata_type_id).calc_script_hash(),
            smt_roots:          new_smt_roots.values().cloned().collect(),
        })
        .as_bytes()
    }
}
