use anyhow::Result;
use ckb_hash::blake2b_256;
use ckb_sdk::traits::SecpCkbRawKeySigner;
use ckb_sdk::unlock::SecpSighashScriptSigner;
use ckb_sdk::{Address, SECP256K1};
use ckb_types::core::ScriptHashType;
use ckb_types::packed::{CellDep, OutPoint, Script, WitnessArgs};
use ckb_types::prelude::{Builder, Pack};
use ckb_types::H256;
use molecule::prelude::Entity;

use common::types::tx_builder::NetworkType;

use crate::ckb::define::scripts::{
    SECP2561_BLAKE160_DEVNET, SECP2561_BLAKE160_MAINNET, SECP2561_BLAKE160_TESTNET,
};
use crate::ckb::NETWORK_TYPE;
use crate::{cell_dep, out_point};

pub struct Sighash {
    pub private_key: H256,
}

impl Sighash {
    pub fn new(private_key: H256) -> Self {
        Self { private_key }
    }

    pub fn address(&self) -> Result<Address> {
        let sender_key = secp256k1::SecretKey::from_slice(self.private_key.as_bytes())?;
        let pubkey = secp256k1::PublicKey::from_secret_key(&SECP256K1, &sender_key);
        let hash160 = blake2b_256(&pubkey.serialize()[..])[0..20].to_vec();

        Ok(match **NETWORK_TYPE.load() {
            NetworkType::Mainnet => {
                let address_payload = ckb_sdk::AddressPayload::new_full(
                    ScriptHashType::Type,
                    SECP2561_BLAKE160_MAINNET.code_hash.clone().pack(),
                    hash160.into(),
                );
                Address::new(ckb_sdk::NetworkType::Mainnet, address_payload, true)
            }
            NetworkType::Testnet => {
                let address_payload = ckb_sdk::AddressPayload::new_full(
                    ScriptHashType::Type,
                    SECP2561_BLAKE160_TESTNET.code_hash.clone().pack(),
                    hash160.into(),
                );
                Address::new(ckb_sdk::NetworkType::Testnet, address_payload, true)
            }
            NetworkType::Devnet => {
                let address_payload = ckb_sdk::AddressPayload::new_full(
                    ScriptHashType::Type,
                    SECP2561_BLAKE160_DEVNET.code_hash.clone().pack(),
                    hash160.into(),
                );
                Address::new(ckb_sdk::NetworkType::Dev, address_payload, true)
            }
        })
    }

    pub fn lock(&self) -> Result<Script> {
        Ok(Script::from(&self.address()?))
    }

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

    pub fn signer(&self) -> Result<SecpSighashScriptSigner> {
        let key = secp256k1::SecretKey::from_slice(self.private_key.as_bytes())?;
        let signer = SecpCkbRawKeySigner::new_with_secret_keys(vec![key]);
        Ok(SecpSighashScriptSigner::new(Box::new(signer)))
    }

    pub fn witness_placeholder() -> WitnessArgs {
        WitnessArgs::new_builder()
            .lock(Some(bytes::Bytes::from(vec![0u8; 65])).pack())
            .build()
    }
}
