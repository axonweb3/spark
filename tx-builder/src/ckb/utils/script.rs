use anyhow::Result;
use axon_types::delegate::DelegateArgs;
use axon_types::selection::SelectionLockArgs;
use axon_types::stake::StakeArgs;
use axon_types::withdraw::WithdrawArgs;
use bytes::Bytes;
use ckb_hash::new_blake2b;
use ckb_sdk::constants::{SIGHASH_TYPE_HASH, TYPE_ID_CODE_HASH};
use ckb_sdk::unlock::OmniLockConfig;
use ckb_types::core::ScriptHashType;
use ckb_types::packed::{Byte32, CellInput, Script};
use ckb_types::prelude::{Builder, Entity, Pack};
use ckb_types::{H160, H256};

use common::types::tx_builder::NetworkType;
use common::utils::convert::{to_axon_byte32, to_identity, to_identity_opt};

use crate::ckb::define::scripts::*;

macro_rules! script {
    ($code_hash: expr, $hash_type: expr, $args: expr) => {
        Script::new_builder()
            .code_hash($code_hash.pack())
            .hash_type($hash_type.into())
            .args($args.pack())
            .build()
    };
}

pub fn omni_eth_lock(network_type: &NetworkType, eth_addr: &H160) -> Script {
    let cfg = OmniLockConfig::new_ethereum(eth_addr.clone());
    let omni_lock_code_hash = match network_type {
        NetworkType::Mainnet => &OMNI_LOCK_MAINNET.code_hash,
        NetworkType::Testnet => &OMNI_LOCK_TESTNET.code_hash,
    };
    script!(omni_lock_code_hash, ScriptHashType::Type, cfg.build_args())
}

pub fn omni_eth_supply_lock(
    network_type: &NetworkType,
    pubkey_hash: H160,
    type_script_hash: Byte32,
) -> Result<Script> {
    let mut cfg = OmniLockConfig::new_ethereum(pubkey_hash);
    cfg.set_info_cell(H256::from_slice(type_script_hash.as_slice()).unwrap());

    let omni_lock_code_hash = match network_type {
        NetworkType::Mainnet => &OMNI_LOCK_MAINNET.code_hash,
        NetworkType::Testnet => &OMNI_LOCK_TESTNET.code_hash,
    };
    Ok(script!(
        omni_lock_code_hash,
        ScriptHashType::Type,
        cfg.build_args()
    ))
}

pub fn sighash_lock(pubkey_hash: &Bytes) -> Script {
    script!(SIGHASH_TYPE_HASH, ScriptHashType::Type, pubkey_hash)
}

pub fn always_success_lock(network_type: &NetworkType) -> Script {
    match network_type {
        NetworkType::Mainnet => script!(
            &ALWAYS_SUCCESS_MAINNET.code_hash,
            ALWAYS_SUCCESS_MAINNET.hash_type,
            bytes::Bytes::default()
        ),
        NetworkType::Testnet => script!(
            &ALWAYS_SUCCESS_TESTNET.code_hash,
            ALWAYS_SUCCESS_TESTNET.hash_type,
            bytes::Bytes::default()
        ),
    }
}

pub fn selection_lock(
    network_type: &NetworkType,
    issue_lock_hash: &Byte32,
    reward_smt_type_id: &Byte32,
) -> Script {
    let selectionn_args = SelectionLockArgs::new_builder()
        .omni_lock_hash(to_axon_byte32(issue_lock_hash))
        .reward_type_id(to_axon_byte32(reward_smt_type_id))
        .build()
        .as_bytes();

    match network_type {
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
    }
}

pub fn type_id(input: &CellInput, output_index: u64) -> H256 {
    let mut blake2b = new_blake2b();

    blake2b.update(input.as_slice());
    blake2b.update(&output_index.to_le_bytes());

    let mut ret = [0; 32];
    blake2b.finalize(&mut ret);

    H256::from_slice(&ret).unwrap()
}

pub fn type_id_script(args: &H256) -> Script {
    let args = Bytes::from(args.as_bytes().to_vec());
    script!(TYPE_ID_CODE_HASH, ScriptHashType::Type, args)
}

pub fn default_type_id() -> Script {
    script!(
        TYPE_ID_CODE_HASH,
        ScriptHashType::Type,
        Bytes::from(vec![0u8; 32])
    )
}

pub fn xudt_type(network_type: &NetworkType, owner_lock_hash: &Byte32) -> Script {
    match network_type {
        NetworkType::Mainnet => script!(
            &XUDT_TYPE_MAINNET.code_hash,
            XUDT_TYPE_MAINNET.hash_type,
            owner_lock_hash.as_bytes()
        ),
        NetworkType::Testnet => script!(
            &XUDT_TYPE_TESTNET.code_hash,
            XUDT_TYPE_TESTNET.hash_type,
            owner_lock_hash.as_bytes()
        ),
    }
}

pub fn checkpoint_type(network_type: &NetworkType, args: &H256) -> Script {
    let args = Bytes::from(args.as_bytes().to_vec());
    match network_type {
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

pub fn metadata_type(network_type: &NetworkType, args: &H256) -> Script {
    let args = Bytes::from(args.as_bytes().to_vec());
    match network_type {
        NetworkType::Mainnet => script!(
            &METADATA_TYPE_MAINNET.code_hash,
            METADATA_TYPE_MAINNET.hash_type,
            args
        ),
        NetworkType::Testnet => script!(
            &METADATA_TYPE_TESTNET.code_hash,
            METADATA_TYPE_TESTNET.hash_type,
            args
        ),
    }
}

pub fn stake_smt_type(network_type: &NetworkType, stake_smt_type_id: &H256) -> Script {
    let args = Bytes::from(stake_smt_type_id.as_bytes().to_vec());
    match network_type {
        NetworkType::Mainnet => script!(&STAKE_MAINNET.code_hash, STAKE_MAINNET.hash_type, args),
        NetworkType::Testnet => script!(&STAKE_TESTNET.code_hash, STAKE_TESTNET.hash_type, args),
    }
}

pub fn stake_lock(
    network_type: &NetworkType,
    metadata_type_id: &H256,
    staker_addr: &H160,
) -> Script {
    // todo: metadata_type(network_type, metadata_type_id).calc_script_hash();
    let metadata_type_hash = type_id_script(metadata_type_id).calc_script_hash();
    let args = StakeArgs::new_builder()
        .metadata_type_id(to_axon_byte32(&metadata_type_hash))
        .stake_addr(to_identity_opt(staker_addr))
        .build()
        .as_bytes();

    match network_type {
        NetworkType::Mainnet => script!(&STAKE_MAINNET.code_hash, STAKE_MAINNET.hash_type, args),
        NetworkType::Testnet => script!(&STAKE_TESTNET.code_hash, STAKE_TESTNET.hash_type, args),
    }
}

pub fn delegate_lock(
    network_type: &NetworkType,
    metadata_type_id: &H256,
    delegate_addr: &H160,
) -> Script {
    // todo: metadata_type(network_type, metadata_type_id).calc_script_hash();
    let metadata_type_hash = type_id_script(metadata_type_id).calc_script_hash();
    let args = DelegateArgs::new_builder()
        .metadata_type_id(to_axon_byte32(&metadata_type_hash))
        .delegator_addr(to_identity_opt(delegate_addr))
        .build()
        .as_bytes();

    match network_type {
        NetworkType::Mainnet => script!(
            &DELEGATE_MAINNET.code_hash,
            DELEGATE_MAINNET.hash_type,
            args
        ),
        NetworkType::Testnet => script!(
            &DELEGATE_TESTNET.code_hash,
            DELEGATE_TESTNET.hash_type,
            args
        ),
    }
}

pub fn delegate_smt_type(network_type: &NetworkType, delegate_smt_type_id: &H256) -> Script {
    let args = Bytes::from(delegate_smt_type_id.as_bytes().to_vec());

    match network_type {
        NetworkType::Mainnet => script!(
            &DELEGATE_MAINNET.code_hash,
            DELEGATE_MAINNET.hash_type,
            args
        ),
        NetworkType::Testnet => script!(
            &DELEGATE_TESTNET.code_hash,
            DELEGATE_TESTNET.hash_type,
            args
        ),
    }
}

pub fn withdraw_lock(network_type: &NetworkType, metadata_type_id: &H256, addr: &H160) -> Script {
    // todo: metadata_type(network_type, metadata_type_id).calc_script_hash();
    let metadata_type_hash = type_id_script(metadata_type_id).calc_script_hash();
    let args = WithdrawArgs::new_builder()
        .metadata_type_id(to_axon_byte32(&metadata_type_hash))
        .addr(to_identity(addr))
        .build()
        .as_bytes();

    match network_type {
        NetworkType::Mainnet => script!(
            &WITHDRAW_LOCK_MAINNET.code_hash,
            WITHDRAW_LOCK_MAINNET.hash_type,
            args
        ),
        NetworkType::Testnet => script!(
            &WITHDRAW_LOCK_TESTNET.code_hash,
            WITHDRAW_LOCK_TESTNET.hash_type,
            args
        ),
    }
}

pub fn reward_smt_type(network_type: &NetworkType, reward_smt_type_id: &H256) -> Script {
    let args = Bytes::from(reward_smt_type_id.as_bytes().to_vec());
    match network_type {
        NetworkType::Mainnet => script!(
            &WITHDRAW_LOCK_MAINNET.code_hash,
            WITHDRAW_LOCK_MAINNET.hash_type,
            args
        ),
        NetworkType::Testnet => script!(
            &WITHDRAW_LOCK_TESTNET.code_hash,
            WITHDRAW_LOCK_TESTNET.hash_type,
            args
        ),
    }
}

pub fn delegate_requirement_type(
    network_type: &NetworkType,
    metadata_type_id: &H256,
    staker_addr: &H160,
) -> Script {
    // todo: args
    let args = metadata_type_id.as_bytes().to_vec();
    args.to_vec().extend(staker_addr.as_bytes().to_vec());
    let args = Bytes::from(args);

    match network_type {
        NetworkType::Mainnet => script!(
            &DELEGATE_REQUIREMENT_TYPE_MAINNET.code_hash,
            DELEGATE_REQUIREMENT_TYPE_MAINNET.hash_type,
            args
        ),
        NetworkType::Testnet => script!(
            &DELEGATE_REQUIREMENT_TYPE_TESTNET.code_hash,
            DELEGATE_REQUIREMENT_TYPE_TESTNET.hash_type,
            args
        ),
    }
}
