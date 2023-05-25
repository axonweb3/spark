use ckb_types::{
    packed::{CellDep, OutPoint},
    prelude::{Builder, Entity, Pack},
};

use common::types::tx_builder::NetworkType;

use crate::ckb::define::script::*;

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
            OMNI_LOCK_MAINNET.tx_hash.clone(),
            OMNI_LOCK_MAINNET.index,
            OMNI_LOCK_MAINNET.dep_type
        ),
        NetworkType::Testnet => cell_dep!(
            OMNI_LOCK_TESTNET.tx_hash.clone(),
            OMNI_LOCK_TESTNET.index,
            OMNI_LOCK_TESTNET.dep_type
        ),
    }
}

pub fn secp256k1_lock_dep(network_type: &NetworkType) -> CellDep {
    match network_type {
        NetworkType::Mainnet => cell_dep!(
            SECP2561_BLAKE160_MAINNET.tx_hash.clone(),
            SECP2561_BLAKE160_MAINNET.index,
            SECP2561_BLAKE160_MAINNET.dep_type
        ),
        NetworkType::Testnet => cell_dep!(
            SECP2561_BLAKE160_TESTNET.tx_hash.clone(),
            SECP2561_BLAKE160_TESTNET.index,
            SECP2561_BLAKE160_TESTNET.dep_type
        ),
    }
}

pub fn xudt_dep(network_type: &NetworkType) -> CellDep {
    match network_type {
        NetworkType::Mainnet => cell_dep!(
            XUDT_MAINNET.tx_hash.clone(),
            XUDT_MAINNET.index,
            XUDT_MAINNET.dep_type
        ),
        NetworkType::Testnet => cell_dep!(
            XUDT_TESTNET.tx_hash.clone(),
            XUDT_TESTNET.index,
            XUDT_TESTNET.dep_type
        ),
    }
}

pub fn selection_dep(network_type: &NetworkType) -> CellDep {
    match network_type {
        NetworkType::Mainnet => cell_dep!(
            SELECTION_LOCK_MAINNET.tx_hash.clone(),
            SELECTION_LOCK_MAINNET.index,
            SELECTION_LOCK_MAINNET.dep_type
        ),
        NetworkType::Testnet => cell_dep!(
            SELECTION_LOCK_TESTNET.tx_hash.clone(),
            SELECTION_LOCK_TESTNET.index,
            SELECTION_LOCK_TESTNET.dep_type
        ),
    }
}

pub fn checkpoint_dep(network_type: &NetworkType) -> CellDep {
    match network_type {
        NetworkType::Mainnet => cell_dep!(
            CHECKPOINT_TYPE_MAINNET.tx_hash.clone(),
            CHECKPOINT_TYPE_MAINNET.index,
            CHECKPOINT_TYPE_MAINNET.dep_type
        ),
        NetworkType::Testnet => cell_dep!(
            CHECKPOINT_TYPE_TESTNET.tx_hash.clone(),
            CHECKPOINT_TYPE_TESTNET.index,
            CHECKPOINT_TYPE_TESTNET.dep_type
        ),
    }
}

pub fn metadata_dep(network_type: &NetworkType) -> CellDep {
    match network_type {
        NetworkType::Mainnet => cell_dep!(
            METADATA_TYPE_MAINNET.tx_hash.clone(),
            METADATA_TYPE_MAINNET.index,
            METADATA_TYPE_MAINNET.dep_type
        ),
        NetworkType::Testnet => cell_dep!(
            METADATA_TYPE_TESTNET.tx_hash.clone(),
            METADATA_TYPE_TESTNET.index,
            METADATA_TYPE_TESTNET.dep_type
        ),
    }
}
