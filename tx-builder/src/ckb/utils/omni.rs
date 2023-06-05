use anyhow::Result;
use ckb_crypto::secp::Pubkey;
use ckb_sdk::traits::SecpCkbRawKeySigner;
use ckb_sdk::types::omni_lock::OmniLockWitnessLock;
use ckb_sdk::unlock::{OmniLockConfig, OmniLockScriptSigner, OmniUnlockMode};
use ckb_sdk::util::keccak160;
use ckb_sdk::{Address, SECP256K1};
use ckb_types::core::ScriptHashType;
use ckb_types::packed::WitnessArgs;
use ckb_types::prelude::{Builder, Pack};
use ckb_types::{H160, H256};
use molecule::prelude::Entity;

use common::types::tx_builder::NetworkType;

use crate::ckb::define::scripts::{OMNI_LOCK_MAINNET, OMNI_LOCK_TESTNET};

pub fn omni_eth_address(private_key: H256) -> Result<H160> {
    let sender_key = secp256k1::SecretKey::from_slice(private_key.as_bytes())?;
    let pubkey = secp256k1::PublicKey::from_secret_key(&SECP256K1, &sender_key);
    let pubkey = Pubkey::from(pubkey);
    Ok(keccak160(pubkey.as_ref()))
}

pub fn omni_eth_config(private_key: H256) -> Result<OmniLockConfig> {
    let addr = omni_eth_address(private_key)?;
    Ok(OmniLockConfig::new_ethereum(addr))
}

pub fn omni_eth_signer(private_key: H256) -> Result<OmniLockScriptSigner> {
    let key = secp256k1::SecretKey::from_slice(private_key.as_bytes())?;
    let signer = SecpCkbRawKeySigner::new_with_ethereum_secret_keys(vec![key]);
    Ok(OmniLockScriptSigner::new(
        Box::new(signer),
        omni_eth_config(private_key)?,
        OmniUnlockMode::Normal,
    ))
}

pub fn omni_eth_ckb_address(network_type: &NetworkType, private_key: H256) -> Result<String> {
    let addr = omni_eth_address(private_key)?;
    let config = OmniLockConfig::new_ethereum(addr);
    Ok(match network_type {
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

pub fn omni_eth_witness_placeholder() -> WitnessArgs {
    let lock_field = OmniLockWitnessLock::new_builder()
        .signature(Some(bytes::Bytes::from(vec![0u8; 65])).pack())
        .build()
        .as_bytes();
    WitnessArgs::new_builder()
        .lock(Some(lock_field).pack())
        .build()
}
