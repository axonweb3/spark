pub mod amount_calculator;
pub mod checkpoint;
pub mod ckb;
pub mod delegate;
pub mod metadata;
pub mod rest;
pub mod stake;
pub mod withdraw;

use anyhow::Result;
use ckb_types::{
    packed::{CellDep, Script as CScript},
    prelude::{Builder, Entity},
};

use common::traits::ckb_rpc_client::CkbRpc;

use crate::ckb::helper::ckb::cell_collector::get_cell_by_type;

pub use checkpoint::Checkpoint;
pub use ckb::*;
pub use delegate::Delegate;
pub use metadata::Metadata;
pub use rest::{Issue, Reward, Selection};
pub use stake::Stake;
pub use withdraw::Withdraw;

#[macro_export]
macro_rules! script {
    ($code_hash: expr, $hash_type: expr, $args: expr) => {
        Script::new_builder()
            .code_hash($code_hash.pack())
            .hash_type($hash_type.into())
            .args($args.pack())
            .build()
    };
}

#[macro_export]
macro_rules! out_point {
    ($tx_hash: expr, $index: expr) => {
        OutPoint::new_builder()
            .tx_hash($tx_hash.pack())
            .index($index.pack())
            .build()
    };
}

#[macro_export]
macro_rules! cell_dep {
    ($tx_hash: expr, $index: expr, $dep_type: expr) => {
        CellDep::new_builder()
            .out_point(out_point!($tx_hash, $index))
            .dep_type($dep_type.into())
            .build()
    };
}

async fn unique_cell_dep(ckb_rpc: &impl CkbRpc, type_id_script: CScript) -> Result<CellDep> {
    let cell = get_cell_by_type(ckb_rpc, type_id_script).await?;

    Ok(CellDep::new_builder()
        .out_point(cell.out_point.into())
        .build())
}

pub fn token_cell_data(amount: u128, extra_args: bytes::Bytes) -> bytes::Bytes {
    let mut res = amount.to_le_bytes().to_vec();
    res.extend(extra_args.to_vec());
    bytes::Bytes::from(res)
}
