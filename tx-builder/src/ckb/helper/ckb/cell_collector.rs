use anyhow::{anyhow, Result};
use ckb_jsonrpc_types::{CellInfo, Uint32};
use ckb_types::{
    packed::{CellInput, OutPoint, Script},
    prelude::*,
};

use common::traits::ckb_rpc_client::CkbRpc;
use common::types::ckb_rpc_client::{Cell, Order, ScriptType, SearchKey, SearchKeyFilter};

use crate::ckb::define::error::*;

pub async fn get_cells(
    ckb_rpc: &impl CkbRpc,
    limit: u32,
    search_key: SearchKey,
) -> Result<Vec<Cell>> {
    let result = ckb_rpc
        .get_cells(search_key.clone(), Order::Asc, Uint32::from(limit), None)
        .await?;

    let mut cells = vec![];
    result.objects.into_iter().for_each(|cell| {
        cells.push(cell);
    });

    Ok(cells)
}

pub async fn get_cell_by_type(ckb_rpc: &impl CkbRpc, type_: Script) -> Result<Cell> {
    let cells = get_cells(ckb_rpc, 1, SearchKey {
        script:               type_.clone().into(),
        script_type:          ScriptType::Type,
        filter:               None,
        script_search_mode:   None,
        with_data:            None,
        group_by_transaction: None,
    })
    .await?;

    if cells.is_empty() {
        return Err(CkbTxErr::CellNotFound(type_.to_string()).into());
    }

    Ok(cells[0].clone())
}

pub async fn get_cell_by_scripts(
    ckb_rpc: &impl CkbRpc,
    lock: Script,
    type_: Script,
) -> Result<Option<Cell>> {
    let cells = get_cells(ckb_rpc, 1, SearchKey {
        script:      lock.into(),
        script_type: ScriptType::Lock,
        filter:      Some(SearchKeyFilter {
            script: Some(type_.into()),
            ..Default::default()
        }),

        script_search_mode:   None,
        with_data:            Some(true),
        group_by_transaction: None,
    })
    .await?;

    if cells.is_empty() {
        Ok(None)
    } else {
        Ok(Some(cells[0].clone()))
    }
}

pub async fn get_live_cells(
    ckb_rpc: &impl CkbRpc,
    search_key: SearchKey,
    mut inputs_capacity: u64,
    outputs_capacity: u64,
) -> Result<(Vec<CellInput>, u64)> {
    let mut inputs = vec![];
    let mut after = None;
    let limit = Uint32::from(20);

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
