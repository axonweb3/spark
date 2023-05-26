use std::collections::HashMap;

use anyhow::Result;
use ckb_jsonrpc_types::{OutputsValidator, Transaction};
use ckb_sdk::types::ScriptGroup;
use ckb_types::{
    core::{Capacity, TransactionView},
    packed::{Byte32, Bytes, CellInput, CellOutput, Script},
    prelude::*,
};

use common::traits::ckb_rpc_client::CkbRpc;
use common::types::ckb_rpc_client::{ScriptType, SearchKey};

use crate::ckb::define::config::FEE_RATE;
use crate::ckb::define::error::CkbTxErr;
use crate::ckb::utils::cell_collector::*;

const KB: u64 = 1000;

/// There is no CKB cell in the input and output of the transaction.
/// Collect CKB cells and add them to the input of the transaction.
/// Add a CKB change cell to the output of the transaction.
pub async fn balance_tx(
    ckb_rpc: &impl CkbRpc,
    capacity_provider: Script,
    tx: TransactionView,
) -> Result<TransactionView> {
    let (tx, outputs_capacity) = add_ckb_to_outputs(tx, capacity_provider.clone())?;

    let (tx, inputs_capacity) =
        add_ckb_to_intputs(ckb_rpc, capacity_provider.clone(), tx, outputs_capacity).await?;

    let tx = change_ckb(tx, inputs_capacity, outputs_capacity)?;
    Ok(tx)
}

pub async fn send_tx(ckb_rpc: &impl CkbRpc, tx: &Transaction) -> Result<String> {
    let outputs_validator = Some(OutputsValidator::Passthrough);
    let tx_hash = ckb_rpc.send_transaction(tx, outputs_validator).await?;
    Ok(tx_hash.to_string())
}

pub struct ScriptGroups {
    pub lock_groups: HashMap<Byte32, ScriptGroup>,
    pub type_groups: HashMap<Byte32, ScriptGroup>,
}

pub async fn gen_script_group(ckb_rpc: &impl CkbRpc, tx: &TransactionView) -> Result<ScriptGroups> {
    #[allow(clippy::mutable_key_type)]
    let mut lock_groups: HashMap<Byte32, ScriptGroup> = HashMap::default();
    #[allow(clippy::mutable_key_type)]
    let mut type_groups: HashMap<Byte32, ScriptGroup> = HashMap::default();

    for (i, input) in tx.inputs().into_iter().enumerate() {
        let output = ckb_rpc
            .get_live_cell(input.previous_output().into(), true)
            .await?;
        let output: CellOutput = output.cell.unwrap().output.into();

        let lock_group_entry = lock_groups
            .entry(output.calc_lock_hash())
            .or_insert_with(|| ScriptGroup::from_lock_script(&output.lock()));
        lock_group_entry.input_indices.push(i);

        if let Some(t) = &output.type_().to_opt() {
            let type_group_entry = type_groups
                .entry(t.calc_script_hash())
                .or_insert_with(|| ScriptGroup::from_type_script(t));
            type_group_entry.input_indices.push(i);
        }
    }

    for (i, output) in tx.outputs().into_iter().enumerate() {
        if let Some(t) = &output.type_().to_opt() {
            let type_group_entry = type_groups
                .entry(t.calc_script_hash())
                .or_insert_with(|| ScriptGroup::from_type_script(t));
            type_group_entry.output_indices.push(i);
        }
    }

    Ok(ScriptGroups {
        lock_groups,
        type_groups,
    })
}

fn add_ckb_to_outputs(
    tx: TransactionView,
    capacity_provider: Script,
) -> Result<(TransactionView, u64)> {
    let mut outputs = tx.outputs().into_iter().collect::<Vec<_>>();
    let mut outputs_data = tx.outputs_data().into_iter().collect::<Vec<_>>();

    outputs.push(
        CellOutput::new_builder()
            .lock(capacity_provider)
            .build_exact_capacity(Capacity::zero())?,
    );

    outputs_data.push(Bytes::default());

    let outputs_capacity = calc_outputs_capacity(&outputs);

    let tx = tx
        .as_advanced_builder()
        .set_outputs(outputs)
        .set_outputs_data(outputs_data)
        .build();

    Ok((tx, outputs_capacity))
}

async fn add_ckb_to_intputs(
    ckb_rpc: &impl CkbRpc,
    capacity_provider: Script,
    tx: TransactionView,
    outputs_capacity: u64,
) -> Result<(TransactionView, u64)> {
    let mut inputs = tx.inputs().into_iter().collect::<Vec<_>>();

    let (mut extra_inputs, inputs_capacity) = fetch_live_cells(
        ckb_rpc,
        SearchKey {
            script:               capacity_provider.into(),
            script_type:          ScriptType::Lock,
            filter:               None,
            script_search_mode:   None,
            with_data:            Some(false),
            group_by_transaction: None,
        },
        calc_inputs_capacity(ckb_rpc, &inputs).await?,
        outputs_capacity + Capacity::bytes(1)?.as_u64(),
    )
    .await?;

    inputs.append(&mut extra_inputs);

    let tx = tx.as_advanced_builder().set_inputs(inputs).build();

    Ok((tx, inputs_capacity))
}

fn change_ckb(
    tx: TransactionView,
    inputs_capacity: u64,
    outputs_capacity: u64,
) -> Result<TransactionView> {
    let tx_size = tx.data().as_reader().serialized_size_in_block();
    let needed_capacity = outputs_capacity + fee(tx_size).as_u64();

    if inputs_capacity < needed_capacity {
        return Err(CkbTxErr::InsufficientCapacity {
            inputs_capacity,
            outputs_capacity: needed_capacity,
        }
        .into());
    }

    let change = inputs_capacity - needed_capacity;

    let mut outputs = tx.outputs().into_iter().collect::<Vec<_>>();
    let idx = outputs.len() - 1;
    let old_capacity: u64 = outputs[idx].capacity().unpack();
    let new_capacity = old_capacity
        .checked_add(change)
        .expect("change cell capacity add overflow");
    outputs[idx] = tx
        .output(idx)
        .expect("last output")
        .as_builder()
        .capacity(new_capacity.pack())
        .build();

    let tx = tx.as_advanced_builder().set_outputs(outputs).build();
    Ok(tx)
}

fn fee(tx_size: usize) -> Capacity {
    let fee = FEE_RATE.saturating_mul(tx_size as u64) / KB;
    Capacity::shannons(fee)
}

async fn calc_inputs_capacity(ckb_rpc: &impl CkbRpc, inputs: &[CellInput]) -> Result<u64> {
    let mut inputs_capacity: u64 = 0;
    for input in inputs.iter() {
        let cell = get_live_cell(ckb_rpc, input.previous_output(), false).await?;
        let output = CellOutput::from(cell.output);
        let input_capacity: u64 = output.capacity().unpack();
        inputs_capacity += input_capacity;
    }
    Ok(inputs_capacity)
}

fn calc_outputs_capacity(outputs: &[CellOutput]) -> u64 {
    outputs
        .iter()
        .map(|output| output.capacity().unpack())
        .collect::<Vec<u64>>()
        .iter()
        .sum::<u64>()
}
