use anyhow::Result;
use ckb_types::{
    packed::{CellDep, OutPoint, Script as CScript},
    prelude::{Builder, Entity, Pack},
    H256,
};

use common::traits::ckb_rpc_client::CkbRpc;
use common::types::tx_builder::NetworkType;

use crate::ckb::define::scripts::*;
use crate::ckb::utils::cell_collector::get_unique_cell;
use crate::ckb::utils::script::{
    checkpoint_type, delegate_smt_type, metadata_type, stake_smt_type,
};

macro_rules! out_point {
    ($tx_hash: expr, $index: expr) => {
        OutPoint::new_builder()
            .tx_hash($tx_hash.pack())
            .index($index.pack())
            .build()
    };
}

macro_rules! cell_dep {
    ($tx_hash: expr, $index: expr, $dep_type: expr) => {
        CellDep::new_builder()
            .out_point(out_point!($tx_hash, $index))
            .dep_type($dep_type.into())
            .build()
    };
}

pub fn omni_lock_dep(network_type: &NetworkType) -> CellDep {
    match network_type {
        NetworkType::Mainnet => cell_dep!(
            &OMNI_LOCK_MAINNET.tx_hash,
            OMNI_LOCK_MAINNET.index,
            OMNI_LOCK_MAINNET.dep_type
        ),
        NetworkType::Testnet => cell_dep!(
            &OMNI_LOCK_TESTNET.tx_hash,
            OMNI_LOCK_TESTNET.index,
            OMNI_LOCK_TESTNET.dep_type
        ),
    }
}

pub fn secp256k1_lock_dep(network_type: &NetworkType) -> CellDep {
    match network_type {
        NetworkType::Mainnet => cell_dep!(
            &SECP2561_BLAKE160_MAINNET.tx_hash,
            SECP2561_BLAKE160_MAINNET.index,
            SECP2561_BLAKE160_MAINNET.dep_type
        ),
        NetworkType::Testnet => cell_dep!(
            &SECP2561_BLAKE160_TESTNET.tx_hash,
            SECP2561_BLAKE160_TESTNET.index,
            SECP2561_BLAKE160_TESTNET.dep_type
        ),
    }
}

pub fn always_success_lock_dep(network_type: &NetworkType) -> CellDep {
    match network_type {
        NetworkType::Mainnet => cell_dep!(
            &ALWAYS_SUCCESS_LOCK_MAINNET.tx_hash,
            ALWAYS_SUCCESS_LOCK_MAINNET.index,
            ALWAYS_SUCCESS_LOCK_MAINNET.dep_type
        ),
        NetworkType::Testnet => cell_dep!(
            &ALWAYS_SUCCESS_LOCK_TESTNET.tx_hash,
            ALWAYS_SUCCESS_LOCK_TESTNET.index,
            ALWAYS_SUCCESS_LOCK_TESTNET.dep_type
        ),
    }
}

pub fn xudt_type_dep(network_type: &NetworkType) -> CellDep {
    match network_type {
        NetworkType::Mainnet => cell_dep!(
            &XUDT_TYPE_MAINNET.tx_hash,
            XUDT_TYPE_MAINNET.index,
            XUDT_TYPE_MAINNET.dep_type
        ),
        NetworkType::Testnet => cell_dep!(
            &XUDT_TYPE_TESTNET.tx_hash,
            XUDT_TYPE_TESTNET.index,
            XUDT_TYPE_TESTNET.dep_type
        ),
    }
}

pub fn selection_lock_dep(network_type: &NetworkType) -> CellDep {
    match network_type {
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
    }
}

pub fn checkpoint_type_dep(network_type: &NetworkType) -> CellDep {
    match network_type {
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

pub fn metadata_type_dep(network_type: &NetworkType) -> CellDep {
    match network_type {
        NetworkType::Mainnet => cell_dep!(
            &METADATA_TYPE_MAINNET.tx_hash,
            METADATA_TYPE_MAINNET.index,
            METADATA_TYPE_MAINNET.dep_type
        ),
        NetworkType::Testnet => cell_dep!(
            &METADATA_TYPE_TESTNET.tx_hash,
            METADATA_TYPE_TESTNET.index,
            METADATA_TYPE_TESTNET.dep_type
        ),
    }
}

pub async fn checkpoint_cell_dep(
    ckb_rpc: &impl CkbRpc,
    network_type: &NetworkType,
    type_id: &H256,
) -> Result<CellDep> {
    unique_cell_dep(ckb_rpc, checkpoint_type(network_type, type_id)).await
}

pub async fn metadata_cell_dep(
    ckb_rpc: &impl CkbRpc,
    network_type: &NetworkType,
    type_id: &H256,
) -> Result<CellDep> {
    unique_cell_dep(ckb_rpc, metadata_type(network_type, type_id)).await
}

pub async fn stake_smt_cell_dep(
    ckb_rpc: &impl CkbRpc,
    network_type: &NetworkType,
    type_id: &H256,
) -> Result<CellDep> {
    unique_cell_dep(ckb_rpc, stake_smt_type(network_type, type_id)).await
}

pub async fn delegate_smt_cell_dep(
    ckb_rpc: &impl CkbRpc,
    network_type: &NetworkType,
    type_id: &H256,
) -> Result<CellDep> {
    unique_cell_dep(ckb_rpc, delegate_smt_type(network_type, type_id)).await
}

async fn unique_cell_dep(ckb_rpc: &impl CkbRpc, type_id_script: CScript) -> Result<CellDep> {
    let cell = get_unique_cell(ckb_rpc, type_id_script).await?;

    Ok(CellDep::new_builder()
        .out_point(cell.out_point.into())
        .build())
}

pub fn stake_lock_dep(network_type: &NetworkType) -> CellDep {
    match network_type {
        NetworkType::Mainnet => cell_dep!(
            &STAKE_LOCK_MAINNET.tx_hash,
            STAKE_LOCK_MAINNET.index,
            STAKE_LOCK_MAINNET.dep_type
        ),
        NetworkType::Testnet => cell_dep!(
            &STAKE_LOCK_TESTNET.tx_hash,
            STAKE_LOCK_TESTNET.index,
            STAKE_LOCK_TESTNET.dep_type
        ),
    }
}

pub fn stake_smt_type_dep(network_type: &NetworkType) -> CellDep {
    match network_type {
        NetworkType::Mainnet => cell_dep!(
            &STAKE_SMT_TYPE_MAINNET.tx_hash,
            STAKE_SMT_TYPE_MAINNET.index,
            STAKE_SMT_TYPE_MAINNET.dep_type
        ),
        NetworkType::Testnet => cell_dep!(
            &STAKE_SMT_TYPE_TESTNET.tx_hash,
            STAKE_SMT_TYPE_TESTNET.index,
            STAKE_SMT_TYPE_TESTNET.dep_type
        ),
    }
}

pub fn delegate_lock_dep(network_type: &NetworkType) -> CellDep {
    match network_type {
        NetworkType::Mainnet => cell_dep!(
            &DELEGATE_LOCK_MAINNET.tx_hash,
            DELEGATE_LOCK_MAINNET.index,
            DELEGATE_LOCK_MAINNET.dep_type
        ),
        NetworkType::Testnet => cell_dep!(
            &DELEGATE_LOCK_TESTNET.tx_hash,
            DELEGATE_LOCK_TESTNET.index,
            DELEGATE_LOCK_TESTNET.dep_type
        ),
    }
}

pub fn delegate_smt_type_dep(network_type: &NetworkType) -> CellDep {
    match network_type {
        NetworkType::Mainnet => cell_dep!(
            &DELEGATE_SMT_TYPE_MAINNET.tx_hash,
            DELEGATE_SMT_TYPE_MAINNET.index,
            DELEGATE_SMT_TYPE_MAINNET.dep_type
        ),
        NetworkType::Testnet => cell_dep!(
            &DELEGATE_SMT_TYPE_TESTNET.tx_hash,
            DELEGATE_SMT_TYPE_TESTNET.index,
            DELEGATE_SMT_TYPE_TESTNET.dep_type
        ),
    }
}

pub fn withdraw_lock_dep(network_type: &NetworkType) -> CellDep {
    match network_type {
        NetworkType::Mainnet => cell_dep!(
            &WITHDRAW_LOCK_MAINNET.tx_hash,
            WITHDRAW_LOCK_MAINNET.index,
            WITHDRAW_LOCK_MAINNET.dep_type
        ),
        NetworkType::Testnet => cell_dep!(
            &WITHDRAW_LOCK_TESTNET.tx_hash,
            WITHDRAW_LOCK_TESTNET.index,
            WITHDRAW_LOCK_TESTNET.dep_type
        ),
    }
}

pub fn delegate_requriement_type_dep(network_type: &NetworkType) -> CellDep {
    match network_type {
        NetworkType::Mainnet => cell_dep!(
            &DELEGATE_REQUIREMENT_TYPE_MAINNET.tx_hash,
            DELEGATE_REQUIREMENT_TYPE_MAINNET.index,
            DELEGATE_REQUIREMENT_TYPE_MAINNET.dep_type
        ),
        NetworkType::Testnet => cell_dep!(
            &DELEGATE_REQUIREMENT_TYPE_TESTNET.tx_hash,
            DELEGATE_REQUIREMENT_TYPE_TESTNET.index,
            DELEGATE_REQUIREMENT_TYPE_TESTNET.dep_type
        ),
    }
}

pub fn reward_smt_type_dep(network_type: &NetworkType) -> CellDep {
    match network_type {
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
    }
}
