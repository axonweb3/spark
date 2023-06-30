use anyhow::Result;
use ckb_crypto::secp::Pubkey;
use ckb_sdk::traits::SecpCkbRawKeySigner;
use ckb_sdk::types::omni_lock::OmniLockWitnessLock;
use ckb_sdk::unlock::{OmniLockConfig, OmniLockScriptSigner, OmniUnlockMode};
use ckb_sdk::util::keccak160;
use ckb_sdk::{Address, SECP256K1};
use ckb_types::core::ScriptHashType;
use ckb_types::packed::{Byte32, CellDep, OutPoint, Script, WitnessArgs};
use ckb_types::prelude::{Builder, Pack};
use ckb_types::{H160, H256};
use molecule::prelude::Entity;

use common::types::tx_builder::NetworkType;

use crate::ckb::define::scripts::{OMNI_LOCK_MAINNET, OMNI_LOCK_TESTNET};
use crate::ckb::NETWORK_TYPE;
use crate::{cell_dep, out_point, script};

pub struct OmniEth {
    pub private_key: H256,
}

impl OmniEth {
    pub fn new(private_key: H256) -> Self {
        Self { private_key }
    }

    pub fn address(&self) -> Result<H160> {
        let sender_key = secp256k1::SecretKey::from_slice(self.private_key.as_bytes())?;
        let pubkey = secp256k1::PublicKey::from_secret_key(&SECP256K1, &sender_key);
        let pubkey = Pubkey::from(pubkey);
        Ok(keccak160(pubkey.as_ref()))
    }

    pub fn config(&self) -> Result<OmniLockConfig> {
        let addr = self.address()?;
        Ok(OmniLockConfig::new_ethereum(addr))
    }

    pub fn signer(&self) -> Result<OmniLockScriptSigner> {
        let key = secp256k1::SecretKey::from_slice(self.private_key.as_bytes())?;
        let signer = SecpCkbRawKeySigner::new_with_ethereum_secret_keys(vec![key]);
        Ok(OmniLockScriptSigner::new(
            Box::new(signer),
            self.config()?,
            OmniUnlockMode::Normal,
        ))
    }

    pub fn ckb_address(&self) -> Result<String> {
        let addr = self.address()?;
        let config = OmniLockConfig::new_ethereum(addr);
        Ok(match **NETWORK_TYPE.load() {
            NetworkType::Mainnet => {
                let address_payload = ckb_sdk::AddressPayload::new_full(
                    ScriptHashType::Type,
                    OMNI_LOCK_MAINNET.code_hash.clone().pack(),
                    config.build_args(),
                );
                Address::new(ckb_sdk::NetworkType::Mainnet, address_payload, true).to_string()
            }
            NetworkType::Testnet => {
                let address_payload = ckb_sdk::AddressPayload::new_full(
                    ScriptHashType::Type,
                    OMNI_LOCK_TESTNET.code_hash.clone().pack(),
                    config.build_args(),
                );
                Address::new(ckb_sdk::NetworkType::Testnet, address_payload, true).to_string()
            }
        })
    }

    pub fn witness_placeholder() -> WitnessArgs {
        let lock_field = OmniLockWitnessLock::new_builder()
            .signature(Some(bytes::Bytes::from(vec![0u8; 65])).pack())
            .build()
            .as_bytes();
        WitnessArgs::new_builder()
            .lock(Some(lock_field).pack())
            .build()
    }

    pub fn lock(eth_addr: &H160) -> Script {
        let cfg = OmniLockConfig::new_ethereum(eth_addr.clone());
        let omni_lock_code_hash = match **NETWORK_TYPE.load() {
            NetworkType::Mainnet => &OMNI_LOCK_MAINNET.code_hash,
            NetworkType::Testnet => &OMNI_LOCK_TESTNET.code_hash,
        };
        script!(omni_lock_code_hash, ScriptHashType::Type, cfg.build_args())
    }

    pub fn supply_lock(pubkey_hash: H160, type_script_hash: Byte32) -> Result<Script> {
        let mut cfg = OmniLockConfig::new_ethereum(pubkey_hash);
        cfg.set_info_cell(H256::from_slice(type_script_hash.as_slice()).unwrap());

        let omni_lock_code_hash = match **NETWORK_TYPE.load() {
            NetworkType::Mainnet => &OMNI_LOCK_MAINNET.code_hash,
            NetworkType::Testnet => &OMNI_LOCK_TESTNET.code_hash,
        };
        Ok(script!(
            omni_lock_code_hash,
            ScriptHashType::Type,
            cfg.build_args()
        ))
    }

    pub fn lock_dep() -> CellDep {
        match **NETWORK_TYPE.load() {
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
}
