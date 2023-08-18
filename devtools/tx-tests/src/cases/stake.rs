use std::fs;
use std::path::Path;

use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;

use crate::config::types::PrivKeys;
use crate::helper::user::get_users;
use crate::tx::*;
use crate::ROCKSDB_PATH;

pub async fn run_stake_case(ckb: &CkbRpcClient, priv_keys: PrivKeys) {
    if Path::new(ROCKSDB_PATH).exists() {
        fs::remove_dir_all(ROCKSDB_PATH).unwrap();
    }

    if priv_keys.staker_privkeys.is_empty() {
        panic!("At least one stakers are required");
    }

    if priv_keys.delegator_privkeys.is_empty() {
        panic!("At least one delegator is required");
    }

    let (stakers_key, _) = get_users(priv_keys.staker_privkeys.clone());
    let staker_key = stakers_key[0].clone();
    let kicker_key = stakers_key[0].clone();

    run_init_tx(ckb, priv_keys.clone()).await;
    run_mint_tx(ckb, priv_keys.clone()).await;

    // staker: +100
    first_stake_tx(ckb, stakers_key[0].clone()).await;
    // wallet: 400, stake: 100, delta: +100

    // Stake too mutch
    assert!(add_stake_tx(ckb, stakers_key[0].clone(), 1000, 0)
        .await
        .is_err());

    // staker: -10
    // When redeeming stake, there are pending records of adding stake with a larger
    // amount
    redeem_stake_tx(ckb, staker_key.clone(), 10, 0)
        .await
        .unwrap();
    // wallet: 410, stake: 90, delta: +90

    // Clear all pending records
    stake_smt_tx(ckb, kicker_key, vec![staker_key.clone()]).await;
    // wallet: 410, stake: 90, delta: 0

    // staker: -10
    redeem_stake_tx(ckb, staker_key.clone(), 10, 0)
        .await
        .unwrap();
    // wallet: 410, stake: 90, delta: -10

    // staker: -10
    // When redeeming stake, there are pending records of redeeming stake
    redeem_stake_tx(ckb, staker_key.clone(), 10, 0)
        .await
        .unwrap();
    // wallet: 410, stake: 90, delta: -20

    // staker: +10
    // When adding stake, there are pending records of redeeming stake with a larger
    // amount
    add_stake_tx(ckb, staker_key.clone(), 10, 0).await.unwrap();
    // wallet: 400, stake: 100, delta: -10

    // staker: +15
    // When adding stake, there are pending records of redeeming stake with a
    // smaller amount
    add_stake_tx(ckb, staker_key.clone(), 15, 0).await.unwrap();
    // wallet: 395, stake: 105, delta: +5

    // staker: -15
    // When redeeming stake, there are pending records of adding stake with a
    // smaller amount
    redeem_stake_tx(ckb, staker_key.clone(), 15, 0)
        .await
        .unwrap();
    // wallet: 400, stake: 100, delta: -10

    // New epoch
    run_checkpoint_tx(ckb, priv_keys.clone(), 1).await;

    // staker: +10
    // When adding stake, there are expired records of redeeming stake
    add_stake_tx(ckb, staker_key.clone(), 10, 1).await.unwrap();
    // wallet: 390, stake: 110, delta: +10

    // New epoch
    run_checkpoint_tx(ckb, priv_keys.clone(), 2).await;

    // staker: +15
    // When adding stake, there are expired records of adding stake with a smaller
    // amount
    add_stake_tx(ckb, staker_key.clone(), 15, 2).await.unwrap();
    // wallet: 385, stake: 115, delta: +5

    // New epoch
    run_checkpoint_tx(ckb, priv_keys.clone(), 3).await;

    // staker: +1
    // When adding stake, there are expired adding stake with a larger amount
    add_stake_tx(ckb, staker_key.clone(), 1, 3).await.unwrap();
    // wallet: 389, stake: 111, delta: 0

    // staker: +10
    add_stake_tx(ckb, staker_key.clone(), 10, 3).await.unwrap();
    // wallet: 379, stake: 121, delta: +10

    // New epoch
    run_checkpoint_tx(ckb, priv_keys.clone(), 4).await;

    // staker: -15
    // When redeeming stake, there are pending records of adding stake with a
    // smaller amount
    redeem_stake_tx(ckb, staker_key.clone(), 15, 4)
        .await
        .unwrap();
    // wallet: 389, stake: 111, delta: -5

    // New epoch
    run_checkpoint_tx(ckb, priv_keys.clone(), 4).await;

    // staker: -1
    // When redeeming stake, there are pending records of redeeming stake
    redeem_stake_tx(ckb, staker_key.clone(), 1, 4)
        .await
        .unwrap();
    // wallet: 389, stake: 111, delta: -1

    // staker: +11
    add_stake_tx(ckb, staker_key.clone(), 11, 3).await.unwrap();
    // wallet: 379, stake: 121, delta: +10

    // New epoch
    run_checkpoint_tx(ckb, priv_keys.clone(), 5).await;

    // staker: -1
    // When redeeming stake, there are pending records of adding stake with a larger
    // amount
    redeem_stake_tx(ckb, staker_key.clone(), 1, 4)
        .await
        .unwrap();
    // wallet: 389,  stake: 111, delta: 0
}
