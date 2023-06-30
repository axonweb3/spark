use anyhow::Result;
use bytes::Bytes;
use ckb_types::packed::{CellDep, OutPoint, Script};
use ckb_types::prelude::{Builder, Entity, Pack};
use ckb_types::H256;

use common::traits::ckb_rpc_client::CkbRpc;
use common::types::ckb_rpc_client::Cell;
use common::types::tx_builder::NetworkType;

use crate::ckb::define::scripts::*;
use crate::ckb::helper::ckb::cell_collector::get_cell_by_type;
use crate::ckb::helper::unique_cell_dep;
use crate::ckb::NETWORK_TYPE;
use crate::{cell_dep, out_point, script};

pub struct Checkpoint;

impl Checkpoint {
    pub fn type_(args: &H256) -> Script {
        let args = Bytes::from(args.as_bytes().to_vec());
        match **NETWORK_TYPE.load() {
            NetworkType::Mainnet => script!(
                &CHECKPOINT_TYPE_MAINNET.code_hash,
                CHECKPOINT_TYPE_MAINNET.hash_type,
                args
            ),
            NetworkType::Testnet => script!(
                &CHECKPOINT_TYPE_TESTNET.code_hash,
                CHECKPOINT_TYPE_TESTNET.hash_type,
                args
            ),
        }
    }

    pub fn type_dep() -> CellDep {
        match **NETWORK_TYPE.load() {
            NetworkType::Mainnet => cell_dep!(
                &CHECKPOINT_TYPE_MAINNET.tx_hash,
                CHECKPOINT_TYPE_MAINNET.index,
                CHECKPOINT_TYPE_MAINNET.dep_type
            ),
            NetworkType::Testnet => cell_dep!(
                &CHECKPOINT_TYPE_TESTNET.tx_hash,
                CHECKPOINT_TYPE_TESTNET.index,
                CHECKPOINT_TYPE_TESTNET.dep_type
            ),
        }
    }

    pub async fn cell_dep(ckb_rpc: &impl CkbRpc, type_id: &H256) -> Result<CellDep> {
        unique_cell_dep(ckb_rpc, Self::type_(type_id)).await
    }

    pub async fn get_cell(ckb_rpc: &impl CkbRpc, checkpoint_type: Script) -> Result<Cell> {
        get_cell_by_type(ckb_rpc, checkpoint_type).await
    }
}
