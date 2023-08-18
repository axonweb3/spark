use std::fs;
use std::path::Path;

use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;

use crate::config::types::PrivKeys;
use crate::helper::user::get_users;
use crate::tx::*;
use crate::ROCKSDB_PATH;

pub async fn run_withdraw_case(ckb: &CkbRpcClient, priv_keys: PrivKeys) {
    if Path::new(ROCKSDB_PATH).exists() {
        fs::remove_dir_all(ROCKSDB_PATH).unwrap();
    }

    if priv_keys.staker_privkeys.len() < 2 {
        panic!("At least 2 stackers are required");
    }

    if priv_keys.delegator_privkeys.len() < 2 {
        panic!("At least 2 delegators are required");
    }

    let (stakers_key, _) = get_users(priv_keys.staker_privkeys.clone());
    let kicker_key = stakers_key[0].clone();
    let staker_key = stakers_key[0].clone();

    run_init_tx(ckb, priv_keys.clone(), 10).await;
    run_mint_tx(ckb, priv_keys.clone()).await;

    // Generate a withdraw cell: (amount: 10, unlock epoch: 2)
    first_stake_tx(ckb, staker_key.clone(), 200).await;
    stake_smt_tx(ckb, kicker_key.clone(), vec![staker_key.clone()]).await;
    redeem_stake_tx(ckb, staker_key.clone(), 10, 0)
        .await
        .unwrap();
    stake_smt_tx(ckb, kicker_key.clone(), vec![staker_key.clone()]).await;

    // New epoch
    run_checkpoint_tx(ckb, priv_keys.clone(), 1).await;

    // Update withdraw cell: (amount: 10, unlock epoch: 2), (amount: 10, unlock
    // epoch: 3)
    redeem_stake_tx(ckb, staker_key.clone(), 10, 1)
        .await
        .unwrap();
    stake_smt_tx(ckb, kicker_key.clone(), vec![staker_key.clone()]).await;

    // New epoch
    run_checkpoint_tx(ckb, priv_keys.clone(), 2).await;
    run_checkpoint_tx(ckb, priv_keys.clone(), 3).await;

    // Withdraw all
    run_withdraw_tx(ckb, staker_key.clone(), 3).await;

    // Update withdraw cell: (amount: 10, unlock epoch: 5)
    redeem_stake_tx(ckb, staker_key.clone(), 10, 3)
        .await
        .unwrap();
    stake_smt_tx(ckb, kicker_key.clone(), vec![staker_key.clone()]).await;

    // New epoch
    run_checkpoint_tx(ckb, priv_keys.clone(), 4).await;

    // Update withdraw cell: (amount: 10, unlock epoch: 5), (amount: 10, unlock
    // epoch: 6)
    redeem_stake_tx(ckb, staker_key.clone(), 10, 3)
        .await
        .unwrap();
    stake_smt_tx(ckb, kicker_key.clone(), vec![staker_key.clone()]).await;

    // New epoch
    run_checkpoint_tx(ckb, priv_keys.clone(), 5).await;

    // Update withdraw cell:
    // (amount: 10, unlock epoch: 5), (amount: 10, unlock epoch: 6), (amount: 10,
    // unlock epoch: 7)
    redeem_stake_tx(ckb, staker_key.clone(), 30, 5)
        .await
        .unwrap();
    stake_smt_tx(ckb, kicker_key.clone(), vec![staker_key.clone()]).await;

    // New epoch
    run_checkpoint_tx(ckb, priv_keys.clone(), 6).await;
    run_checkpoint_tx(ckb, priv_keys.clone(), 7).await;
    run_checkpoint_tx(ckb, priv_keys.clone(), 8).await;

    // Withdraw all
    run_withdraw_tx(ckb, staker_key.clone(), 8).await;
}
