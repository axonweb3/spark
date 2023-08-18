use std::fs;
use std::path::Path;

use anyhow::Result;
use ckb_types::H256;
use common::types::tx_builder::{DelegateItem, EthAddress};
use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;

use crate::config::types::PrivKeys;
use crate::helper::user::get_users;
use crate::tx::*;
use crate::ROCKSDB_PATH;

pub async fn run_metadata_case(ckb: &CkbRpcClient, priv_keys: PrivKeys) {
    if Path::new(ROCKSDB_PATH).exists() {
        fs::remove_dir_all(ROCKSDB_PATH).unwrap();
    }

    if priv_keys.staker_privkeys.len() < 4 {
        panic!("At least 4 stackers are required");
    }

    let (stakers_key, stakers) = get_users(priv_keys.staker_privkeys.clone());
    let (delegators_key, _) = get_users(priv_keys.staker_privkeys.clone());
    let kicker_key = stakers_key[0].clone();

    run_init_tx(ckb, priv_keys.clone(), 1).await;
    run_mint_tx(ckb, priv_keys.clone()).await;

    // staker1: 10
    first_stake_tx(ckb, stakers_key[0].clone(), 10).await;
    // staker2: 20
    first_stake_tx(ckb, stakers_key[1].clone(), 20).await;
    // staker3: 30
    first_stake_tx(ckb, stakers_key[2].clone(), 30).await;

    // delegator: (staker1, +10), (staker2, +10), (staker3, +10), (staker4, +10)
    first_delegates_tx(ckb, delegators_key[0].clone(), &stakers)
        .await
        .unwrap();

    stake_smt_tx(ckb, kicker_key.clone(), stakers_key.clone()).await;
    // staker3: 30
    // staker2: 20
    // staker1: 10

    delegate_smt_tx(ckb, kicker_key.clone(), delegators_key).await;
    // staker1: (delegator1, 10)
    // staker2: (delegator1, 10)
    // staker3: (delegator1, 10)

    // quorum = 1
    // (staker1, 10): (delegator1, 10)
    // (staker2, 20): (delegator1, 10)
    // (staker3, 30): (delegator1, 10)
    // Remove staker1 and staker2 from the stake smt
    // The delegator1's refunded amount should be added up to 20
    run_metadata_tx(ckb, kicker_key.clone()).await;
}

async fn first_delegates_tx(
    ckb: &CkbRpcClient,
    delegator_key: H256,
    stakers: &[EthAddress],
) -> Result<()> {
    println!("first delegate");

    let mut delegates = vec![];

    for staker in stakers.iter() {
        delegates.push(DelegateItem {
            staker:             staker.clone(),
            is_increase:        true,
            amount:             10,
            inauguration_epoch: 2,
        });
    }

    delegate_tx(ckb, delegator_key, delegates, 0, true).await
}
