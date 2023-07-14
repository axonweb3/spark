use bytes::Bytes;
use ckb_hash::new_blake2b;
use ckb_sdk::constants::TYPE_ID_CODE_HASH;
use ckb_types::core::ScriptHashType;
use ckb_types::packed::{CellDep, CellInput, OutPoint, Script};
use ckb_types::prelude::{Builder, Entity, Pack};
use ckb_types::H256;

use common::types::tx_builder::NetworkType;

use crate::ckb::define::scripts::*;
use crate::ckb::NETWORK_TYPE;
use crate::{cell_dep, out_point, script};

pub struct AlwaysSuccess;
pub struct Secp256k1;
pub struct TypeId;

impl AlwaysSuccess {
    pub fn lock() -> Script {
        match **NETWORK_TYPE.load() {
            NetworkType::Mainnet => script!(
                &ALWAYS_SUCCESS_LOCK_MAINNET.code_hash,
                ALWAYS_SUCCESS_LOCK_MAINNET.hash_type,
                bytes::Bytes::default()
            ),
            NetworkType::Testnet => script!(
                &ALWAYS_SUCCESS_LOCK_TESTNET.code_hash,
                ALWAYS_SUCCESS_LOCK_TESTNET.hash_type,
                bytes::Bytes::default()
            ),
            NetworkType::Devnet => script!(
                &ALWAYS_SUCCESS_LOCK_DEVNET.code_hash,
                ALWAYS_SUCCESS_LOCK_DEVNET.hash_type,
                bytes::Bytes::default()
            ),
        }
    }

    pub fn lock_dep() -> CellDep {
        match **NETWORK_TYPE.load() {
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
            NetworkType::Devnet => cell_dep!(
                &ALWAYS_SUCCESS_LOCK_DEVNET.tx_hash,
                ALWAYS_SUCCESS_LOCK_DEVNET.index,
                ALWAYS_SUCCESS_LOCK_DEVNET.dep_type
            ),
        }
    }
}

impl Secp256k1 {
    pub fn lock_dep() -> CellDep {
        match **NETWORK_TYPE.load() {
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
            NetworkType::Devnet => cell_dep!(
                &SECP2561_BLAKE160_DEVNET.tx_hash,
                SECP2561_BLAKE160_DEVNET.index,
                SECP2561_BLAKE160_DEVNET.dep_type
            ),
        }
    }
}

impl TypeId {
    pub fn calc(input: &CellInput, output_index: u64) -> H256 {
        let mut blake2b = new_blake2b();

        blake2b.update(input.as_slice());
        blake2b.update(&output_index.to_le_bytes());

        let mut ret = [0; 32];
        blake2b.finalize(&mut ret);

        H256::from_slice(&ret).unwrap()
    }

    pub fn script(args: &H256) -> Script {
        let args = Bytes::from(args.as_bytes().to_vec());
        script!(TYPE_ID_CODE_HASH, ScriptHashType::Type, args)
    }

    pub fn mock() -> Script {
        script!(
            TYPE_ID_CODE_HASH,
            ScriptHashType::Type,
            Bytes::from(vec![0u8; 32])
        )
    }
}
