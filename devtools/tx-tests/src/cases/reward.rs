use std::fs;
use std::path::Path;

use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;

use crate::config::types::PrivKeys;
use crate::helper::user::gen_users;
use crate::tx::*;
use crate::ROCKSDB_PATH;

pub async fn run_reward_case(ckb: &CkbRpcClient, priv_keys: PrivKeys) {
    if Path::new(ROCKSDB_PATH).exists() {
        fs::remove_dir_all(ROCKSDB_PATH).unwrap();
    }

    if priv_keys.staker_privkeys.is_empty() {
        panic!("At least one stakers are required");
    }

    if priv_keys.delegator_privkeys.len() < 2 {
        panic!("At least 2 delegators are required");
    }

    let seeder_key = priv_keys.seeder_privkey.clone().into_h256().unwrap();
    let (stakers_key, stakers) = gen_users(priv_keys.staker_privkeys.clone());
    let (delegators_key, _) = gen_users(priv_keys.delegator_privkeys.clone());
    let kicker_key = stakers_key[0].clone();

    let staker = stakers[0].clone();
    let staker_key = stakers_key[0].clone();
    let stakers_key = vec![staker_key.clone()];
    let delegator_key = delegators_key[0].clone();
    let delegator_key1 = delegators_key[1].clone();
    let delegators_key = vec![delegator_key.clone()];

    run_init_tx(ckb, seeder_key, stakers_key.clone(), 10).await;
    run_mint_tx(ckb, priv_keys.clone()).await;

    for key in stakers_key.iter() {
        first_stake_tx(ckb, key.clone(), 100).await;
    }
    stake_smt_tx(ckb, kicker_key.clone(), stakers_key.clone(), 0).await;

    first_delegate_tx(ckb, delegator_key.clone(), staker.clone())
        .await
        .unwrap();
    delegate_smt_tx(ckb, kicker_key.clone(), delegators_key.clone(), 0).await;

    run_metadata_tx(ckb, kicker_key.clone()).await;

    run_checkpoint_tx(ckb, kicker_key.clone(), stakers_key.clone(), 1).await;
    run_metadata_tx(ckb, kicker_key.clone()).await;

    run_checkpoint_tx(ckb, kicker_key.clone(), stakers_key.clone(), 2).await;
    run_metadata_tx(ckb, kicker_key.clone()).await;

    println!("reward should failed");
    assert!(run_reward_tx(ckb, staker_key.clone(), 3).await.is_err());

    // Validator
    println!("validator claims rewards");
    run_reward_tx(ckb, staker_key.clone(), 4).await.unwrap();

    // Claim the reward again
    println!("validator claims rewards again");
    assert!(run_reward_tx(ckb, staker_key.clone(), 4).await.is_err());

    // Delegator
    println!("delegator claims rewards");
    run_reward_tx(ckb, delegator_key.clone(), 4).await.unwrap();

    // Neither a validator nor delegator
    println!("neither a validator nor delegator claims rewards");
    run_reward_tx(ckb, delegator_key1, 4).await.unwrap();
}
