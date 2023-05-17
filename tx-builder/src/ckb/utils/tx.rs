use anyhow::{anyhow, Result};
use ckb_jsonrpc_types::{CellInfo, OutputsValidator, Transaction, Uint32};
use ckb_types::{
    core::{Capacity, TransactionView},
    packed::{CellInput, CellOutput, OutPoint, Script},
    prelude::*,
};

use common::traits::ckb_rpc_client::CkbRpc;
use common::types::ckb_rpc_client::{Order, ScriptType, SearchKey};

use crate::ckb::define::config::FEE_RATE;
use crate::ckb::define::error::CkbTxErr;

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
    let mut outputs_capacity = outputs
        .iter()
        .map(|output| output.capacity().unpack())
        .collect::<Vec<u64>>()
        .iter()
        .sum::<u64>();

    let tx_size = tx.data().as_reader().serialized_size_in_block();
    outputs_capacity += fee(tx_size).as_u64();

    let search = SearchKey {
        script:               capacity_provider.into(),
        script_type:          ScriptType::Lock,
        filter:               None,
        script_search_mode:   None,
        with_data:            Some(false),
        group_by_transaction: None,
    };
    let (mut extra_inputs, inputs_capacity) =
        fetch_live_cells(ckb_rpc, search, inputs_capacity, outputs_capacity).await?;
    inputs.append(&mut extra_inputs);

    let tx = tx.as_advanced_builder().set_inputs(inputs).build();

    if inputs_capacity < outputs_capacity {
        return Err(CkbTxErr::InsufficientCapacity {
            inputs_capacity,
            outputs_capacity,
        }
        .into());
    }
    let change = inputs_capacity - outputs_capacity;

    let mut outputs = tx.outputs().into_iter().collect::<Vec<_>>();
    let idx = outputs.len() - 1;
    outputs[idx] = tx
        .output(idx)
        .expect("last output")
        .as_builder()
        .capacity(change.pack())
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

async fn fetch_live_cells(
    ckb_rpc: &impl CkbRpc,
    search_key: SearchKey,
    mut inputs_capacity: u64,
    outputs_capacity: u64,
) -> Result<(Vec<CellInput>, u64)> {
    let mut inputs = vec![];
    let mut after = None;
    let limit = Uint32::from(100000000);
    while inputs_capacity < outputs_capacity {
        let result = ckb_rpc
            .get_cells(search_key.clone(), Order::Asc, limit, after)
            .await?;
        result
            .objects
            .into_iter()
            .filter(|cell| {
                cell.output.type_.is_none()
                    && (cell.output_data.is_none() || cell.output_data.as_ref().unwrap().is_empty())
            })
            .for_each(|cell| {
                if inputs_capacity < outputs_capacity {
                    inputs.push(
                        CellInput::new_builder()
                            .previous_output(cell.out_point.into())
                            .build(),
                    );
                    inputs_capacity += u64::from(cell.output.capacity);
                }
            });
        if result.last_cursor.is_empty() {
            break;
        }
        after = Some(result.last_cursor);
    }
    Ok((inputs, inputs_capacity))
}

async fn get_live_cell(
    ckb_rpc: &impl CkbRpc,
    out_point: OutPoint,
    with_data: bool,
) -> Result<CellInfo> {
    let cell = ckb_rpc
        .get_live_cell(out_point.clone().into(), with_data)
        .await?;
    if cell.status != "live" {
        return Err(anyhow!(
            "Invalid cell status: {}, out_point: {}",
            cell.status,
            out_point,
        ));
    }
    Ok(cell.cell.unwrap())
}
