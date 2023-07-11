use anyhow::Result;
use bytes::Bytes;
use ckb_types::packed::{Byte32, CellDep, OutPoint, Script};
use ckb_types::prelude::{Builder, Entity, Pack};
use ckb_types::H256;

use common::traits::ckb_rpc_client::CkbRpc;
use common::types::axon_types::selection::SelectionLockArgs;
use common::types::ckb_rpc_client::Cell;
use common::types::tx_builder::NetworkType;
use common::utils::convert::*;

use crate::ckb::define::scripts::*;
use crate::ckb::helper::ckb::cell_collector::get_cell_by_type;
use crate::ckb::helper::TypeId;
use crate::ckb::NETWORK_TYPE;
use crate::{cell_dep, out_point, script};

pub struct Issue;
pub struct Selection;
pub struct Reward;

impl Issue {
    pub fn type_(issue_type_id: &H256) -> Script {
        TypeId::script(issue_type_id)
    }

    pub async fn get_cell(ckb_rpc: &impl CkbRpc, reward_type_id: &H256) -> Result<Cell> {
        get_cell_by_type(ckb_rpc, Self::type_(reward_type_id)).await
    }
}

impl Selection {
    pub fn type_(selection_type_id: &H256) -> Script {
        TypeId::script(selection_type_id)
    }

    pub fn lock(issue_lock_hash: &Byte32, reward_smt_type_id: &Byte32) -> Script {
        let selectionn_args = SelectionLockArgs::new_builder()
            .omni_lock_hash(to_axon_byte32(issue_lock_hash))
            .reward_type_id(to_axon_byte32(reward_smt_type_id))
            .build()
            .as_bytes();

        match **NETWORK_TYPE.load() {
            NetworkType::Mainnet => script!(
                &SELECTION_LOCK_MAINNET.code_hash,
                SELECTION_LOCK_MAINNET.hash_type,
                selectionn_args
            ),
            NetworkType::Testnet => script!(
                &SELECTION_LOCK_TESTNET.code_hash,
                SELECTION_LOCK_TESTNET.hash_type,
                selectionn_args
            ),
            NetworkType::Devnet => script!(
                &SELECTION_LOCK_DEVNET.code_hash,
                SELECTION_LOCK_DEVNET.hash_type,
                selectionn_args
            ),
        }
    }

    pub fn lock_dep() -> CellDep {
        match **NETWORK_TYPE.load() {
            NetworkType::Mainnet => cell_dep!(
                &SELECTION_LOCK_MAINNET.tx_hash,
                SELECTION_LOCK_MAINNET.index,
                SELECTION_LOCK_MAINNET.dep_type
            ),
            NetworkType::Testnet => cell_dep!(
                &SELECTION_LOCK_TESTNET.tx_hash,
                SELECTION_LOCK_TESTNET.index,
                SELECTION_LOCK_TESTNET.dep_type
            ),
            NetworkType::Devnet => cell_dep!(
                &SELECTION_LOCK_DEVNET.tx_hash,
                SELECTION_LOCK_DEVNET.index,
                SELECTION_LOCK_DEVNET.dep_type
            ),
        }
    }

    pub async fn get_cell(ckb_rpc: &impl CkbRpc, selection_type_id: &H256) -> Result<Cell> {
        get_cell_by_type(ckb_rpc, Self::type_(selection_type_id)).await
    }
}

impl Reward {
    pub fn smt_type(reward_smt_type_id: &H256) -> Script {
        let args = Bytes::from(reward_smt_type_id.as_bytes().to_vec());
        match **NETWORK_TYPE.load() {
            NetworkType::Mainnet => script!(
                &REWARD_SMT_TYPE_MAINNET.code_hash,
                REWARD_SMT_TYPE_MAINNET.hash_type,
                args
            ),
            NetworkType::Testnet => script!(
                &REWARD_SMT_TYPE_TESTNET.code_hash,
                REWARD_SMT_TYPE_TESTNET.hash_type,
                args
            ),
            NetworkType::Devnet => script!(
                &REWARD_SMT_TYPE_DEVNET.code_hash,
                REWARD_SMT_TYPE_DEVNET.hash_type,
                args
            ),
        }
    }

    pub fn smt_type_dep() -> CellDep {
        match **NETWORK_TYPE.load() {
            NetworkType::Mainnet => cell_dep!(
                &REWARD_SMT_TYPE_MAINNET.tx_hash,
                REWARD_SMT_TYPE_MAINNET.index,
                REWARD_SMT_TYPE_MAINNET.dep_type
            ),
            NetworkType::Testnet => cell_dep!(
                &REWARD_SMT_TYPE_TESTNET.tx_hash,
                REWARD_SMT_TYPE_TESTNET.index,
                REWARD_SMT_TYPE_TESTNET.dep_type
            ),
            NetworkType::Devnet => cell_dep!(
                &REWARD_SMT_TYPE_DEVNET.tx_hash,
                REWARD_SMT_TYPE_DEVNET.index,
                REWARD_SMT_TYPE_DEVNET.dep_type
            ),
        }
    }

    pub async fn get_cell(ckb_rpc: &impl CkbRpc, reward_type_id: &H256) -> Result<Cell> {
        get_cell_by_type(ckb_rpc, Self::smt_type(reward_type_id)).await
    }
}
