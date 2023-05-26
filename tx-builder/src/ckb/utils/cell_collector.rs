use anyhow::{anyhow, Result};
use ckb_jsonrpc_types::{CellInfo, Uint32};
use ckb_types::{
    packed::{CellInput, OutPoint},
    prelude::*,
};

use common::traits::ckb_rpc_client::CkbRpc;
use common::types::ckb_rpc_client::{Cell, Order, SearchKey};

pub async fn fetch_live_cells(
    ckb_rpc: &impl CkbRpc,
    search_key: SearchKey,
    mut inputs_capacity: u64,
    outputs_capacity: u64,
) -> Result<(Vec<CellInput>, u64)> {
    let mut inputs = vec![];
    let mut after = None;
    let limit = Uint32::from(100);
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

pub async fn get_live_cell(
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

pub async fn collect_cells(ckb_rpc: &impl CkbRpc, search_key: SearchKey) -> Result<Vec<Cell>> {
    let result = ckb_rpc
        .get_cells(search_key.clone(), Order::Asc, Uint32::from(100), None)
        .await?;

    let mut inputs = vec![];
    result.objects.into_iter().for_each(|cell| {
        inputs.push(cell);
    });

    Ok(inputs)
}
