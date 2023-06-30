use bytes::Bytes;
use molecule::prelude::Entity;
use ophelia::PublicKey;
use ophelia_blst::BlsPublicKey;
use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;

use common::traits::tx_builder::IStakeTxBuilder;
use common::types::axon_types::basic::{Byte48, Byte65};
use common::types::tx_builder::{DelegateRequirement, FirstStakeInfo, StakeItem, StakeTypeIds};
use tx_builder::ckb::helper::{OmniEth, Tx};
use tx_builder::ckb::stake::StakeTxBuilder;

use crate::config::parse_file;
use crate::config::types::{PrivKeys, TypeIds as CTypeIds};
use crate::{PRIV_KEYS_PATH, TYPE_IDS_PATH};

pub async fn first_stake_tx(ckb: &CkbRpcClient) {
    println!("first stake");

    let (l1_pub_key, bls_pub_key) = gen_pubkey();
    stake_tx(
        ckb,
        StakeItem {
            is_increase:        true,
            amount:             10,
            inauguration_epoch: 2,
        },
        0,
        Some(FirstStakeInfo {
            l1_pub_key,
            bls_pub_key,
            delegate: DelegateRequirement {
                commission_rate:    80,
                maximum_delegators: 2,
                threshold:          0,
            },
        }),
    )
    .await;
}

pub async fn add_stake_tx(ckb: &CkbRpcClient) {
    println!("add stake");

    stake_tx(
        ckb,
        StakeItem {
            is_increase:        true,
            amount:             1,
            inauguration_epoch: 2,
        },
        0,
        None,
    )
    .await;
}

pub async fn reedem_stake_tx(ckb: &CkbRpcClient) {
    println!("redeem stake");

    stake_tx(
        ckb,
        StakeItem {
            is_increase:        false,
            amount:             3,
            inauguration_epoch: 2,
        },
        0,
        None,
    )
    .await;
}

async fn stake_tx(
    ckb: &CkbRpcClient,
    stake_item: StakeItem,
    current_epoch: u64,
    first_stake_info: Option<FirstStakeInfo>,
) {
    let priv_keys: PrivKeys = parse_file(PRIV_KEYS_PATH);
    let test_staker_key = priv_keys.staker_privkeys[0].clone().into_h256().unwrap();
    let omni_eth = OmniEth::new(test_staker_key.clone());
    println!("staker ckb addres: {}\n", omni_eth.ckb_address().unwrap());

    let first_stake = first_stake_info.is_some();
    let type_ids: CTypeIds = parse_file(TYPE_IDS_PATH);
    let checkpoint_type_id = type_ids.checkpoint_type_id.into_h256().unwrap();
    let metadata_type_id = type_ids.metadata_type_id.into_h256().unwrap();
    let xudt_args = type_ids.xudt_owner.into_h256().unwrap();

    let tx = StakeTxBuilder::new(
        ckb,
        StakeTypeIds {
            metadata_type_id,
            checkpoint_type_id,
            xudt_owner: xudt_args,
        },
        omni_eth.address().unwrap(),
        current_epoch,
        stake_item,
        first_stake_info,
    )
    .build_tx()
    .await
    .unwrap();

    let mut tx = Tx::new(ckb, tx);
    let script_groups = tx.gen_script_group().await.unwrap();
    let signer = omni_eth.signer().unwrap();

    if first_stake {
        for group in script_groups.lock_groups.iter() {
            tx.sign(&signer, group.1).unwrap();
        }
    } else {
        let mut first_group = true;
        for group in script_groups.lock_groups.iter() {
            if !first_group {
                println!("sign; not stake id: {:?}", group.1.input_indices);
                tx.sign(&signer, group.1).unwrap();
            } else {
                println!("not sign; stake id: {:?}", group.1.input_indices);
            }
            first_group = false;
        }
    }

    match tx.send().await {
        Ok(tx_hash) => println!("tx hash: 0x{}", tx_hash),
        Err(e) => println!("{}", e),
    }

    println!("\ntx: {}", tx.inner());
}

fn hex_decode(src: &str) -> Vec<u8> {
    if src.is_empty() {
        return Vec::new();
    }

    let src = if src.starts_with("0x") {
        src.split_at(2).1
    } else {
        src
    };

    let src = src.as_bytes();
    let mut ret = vec![0u8; src.len() / 2];
    faster_hex::hex_decode(src, &mut ret).unwrap();

    ret
}

fn gen_pubkey() -> (Byte65, Byte48) {
    let pub_key =
        hex_decode("ac85bbb40347b6e06ac2dc2da1f75eece029cdc0ed2d456c457d27e288bfbfbcd4c5c19716e9b250134a0e76ce50fa22");
    let bls_public_key: BlsPublicKey = BlsPublicKey::try_from(pub_key.as_ref()).unwrap();
    (
        Byte65::new_unchecked(Bytes::from(pub_key)),
        Byte48::new_unchecked(bls_public_key.to_bytes()),
    )
}

#[test]
fn bls_pub_key() {
    let (pub_key, bls_pub_key) = gen_pubkey();
    println!("pub key len: {:?}", pub_key.as_bytes().len());
    println!("bls pub key len: {}", bls_pub_key.as_bytes().len());
}
