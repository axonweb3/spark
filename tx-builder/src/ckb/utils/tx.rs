use anyhow::Result;
use ckb_jsonrpc_types::{OutputsValidator, Transaction};
use ckb_types::{
    core::{Capacity, TransactionView},
    packed::{CellInput, CellOutput, Script},
    prelude::*,
};

use common::traits::ckb_rpc_client::CkbRpc;
use common::types::ckb_rpc_client::{ScriptType, SearchKey};

use crate::ckb::define::config::FEE_RATE;
use crate::ckb::define::error::CkbTxErr;
use crate::ckb::utils::cell_collector::*;

const KB: u64 = 1000;

// The last cell of outputs is CKB change cell.
// There is no CKB cell in inputs.
pub async fn balance_tx(
    ckb_rpc: &impl CkbRpc,
    capacity_provider: Script,
    tx: TransactionView,
) -> Result<TransactionView> {
    let mut inputs = tx.inputs().into_iter().collect::<Vec<_>>();
    let inputs_capacity = calc_inputs_capacity(ckb_rpc, &inputs).await?;

    let outputs = tx.outputs().into_iter().collect::<Vec<_>>();
    let outputs_capacity = calc_outputs_capacity(&outputs);

    let search = SearchKey {
        script:               capacity_provider.into(),
        script_type:          ScriptType::Lock,
        filter:               None,
        script_search_mode:   None,
        with_data:            Some(false),
        group_by_transaction: None,
    };
    let (mut extra_inputs, inputs_capacity) = fetch_live_cells(
        ckb_rpc,
        search,
        inputs_capacity,
        outputs_capacity + Capacity::bytes(1)?.as_u64(),
    )
    .await?;
    inputs.append(&mut extra_inputs);

    let tx = tx.as_advanced_builder().set_inputs(inputs).build();

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

pub async fn send_tx(ckb_rpc: &impl CkbRpc, tx: &Transaction) -> Result<String> {
    let outputs_validator = Some(OutputsValidator::Passthrough);
    let tx_hash = ckb_rpc.send_transaction(tx, outputs_validator).await?;
    Ok(tx_hash.to_string())
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
