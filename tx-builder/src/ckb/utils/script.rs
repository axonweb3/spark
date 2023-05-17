use ckb_sdk::unlock::OmniLockConfig;
use ckb_types::core::ScriptHashType;
use ckb_types::packed::Script;
use ckb_types::prelude::{Builder, Entity, Pack};
use ckb_types::{H160, H256};

use common::types::tx_builder::NetworkType;

use crate::ckb::define::script::*;

macro_rules! script {
    ($code_hash: expr, $hash_type: expr, $args: expr) => {
        Script::new_builder()
            .code_hash($code_hash.pack())
            .hash_type($hash_type.into())
            .args($args.pack())
            .build()
    };
}

pub fn omni_eth_lock(network_type: &NetworkType, addr: &H160) -> Script {
    let cfg = OmniLockConfig::new_ethereum(addr.clone());
    let omni_lock_code_hash = match network_type {
        NetworkType::Mainnet => OMNI_LOCK_MAINNET.code_hash.clone(),
        NetworkType::Testnet => OMNI_LOCK_TESTNET.code_hash.clone(),
    };
    script!(omni_lock_code_hash, ScriptHashType::Type, cfg.build_args())
}

pub fn _cannot_destroy_lock(network_type: &NetworkType) -> Script {
    match network_type {
        NetworkType::Mainnet => script!(
            CANNOT_DESTROY_MAINNET.code_hash.clone(),
            CANNOT_DESTROY_MAINNET.hash_type,
            bytes::Bytes::default()
        ),
        NetworkType::Testnet => script!(
            CANNOT_DESTROY_TESTNET.code_hash.clone(),
            CANNOT_DESTROY_TESTNET.hash_type,
            bytes::Bytes::default()
        ),
    }
}

pub fn selection_lock(lock_hash: &H256, issue_type_id: H256, reward_smt_type_id: H256) -> Script {
    let mut args = issue_type_id.as_bytes().to_vec();
    args.extend(reward_smt_type_id.as_bytes());
    script!(lock_hash, ScriptHashType::Data, args) // todo: ScriptHashType::Type
}
