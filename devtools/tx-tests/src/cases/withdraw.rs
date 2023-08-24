use std::fs;
use std::path::Path;

use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;

use crate::config::types::PrivKeys;
use crate::helper::user::gen_users;
use crate::tx::*;
use crate::ROCKSDB_PATH;

pub async fn run_withdraw_case(ckb: &CkbRpcClient, priv_keys: PrivKeys) {
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
    let (stakers_key, _) = gen_users(priv_keys.staker_privkeys.clone());
    let kicker_key = stakers_key[0].clone();
    let staker_key = stakers_key[0].clone();
    let stakers_key = vec![staker_key.clone()];

    run_init_tx(ckb, seeder_key, stakers_key.clone(), 10).await;
    run_mint_tx(ckb, priv_keys.clone()).await;

    // Generate a withdraw cell: (amount: 10, unlock epoch: 2)
    first_stake_tx(ckb, staker_key.clone(), 200).await;
    stake_smt_tx(ckb, kicker_key.clone(), stakers_key.clone(), 0).await;
    redeem_stake_tx(ckb, staker_key.clone(), 10, 0)
        .await
        .unwrap();
    stake_smt_tx(ckb, kicker_key.clone(), stakers_key.clone(), 0).await;

    // New epoch
    run_metadata_tx(ckb, kicker_key.clone()).await;
    run_checkpoint_tx(ckb, kicker_key.clone(), stakers_key.clone(), 1).await;

    // Update withdraw cell: (amount: 10, unlock epoch: 2), (amount: 10, unlock
    // epoch: 3)
    redeem_stake_tx(ckb, staker_key.clone(), 10, 1)
        .await
        .unwrap();
    stake_smt_tx(ckb, kicker_key.clone(), stakers_key.clone(), 1).await;

    // New epoch
    run_metadata_tx(ckb, kicker_key.clone()).await;
    run_checkpoint_tx(ckb, kicker_key.clone(), stakers_key.clone(), 2).await;
    run_metadata_tx(ckb, kicker_key.clone()).await;
    run_checkpoint_tx(ckb, kicker_key.clone(), stakers_key.clone(), 3).await;

    // Withdraw all
    run_withdraw_tx(ckb, staker_key.clone(), 3).await;

    // Update withdraw cell: (amount: 10, unlock epoch: 5)
    redeem_stake_tx(ckb, staker_key.clone(), 10, 3)
        .await
        .unwrap();
    stake_smt_tx(ckb, kicker_key.clone(), stakers_key.clone(), 3).await;

    // New epoch
    run_metadata_tx(ckb, kicker_key.clone()).await;
    run_checkpoint_tx(ckb, kicker_key.clone(), stakers_key.clone(), 4).await;

    // Update withdraw cell: (amount: 10, unlock epoch: 5), (amount: 10, unlock
    // epoch: 6)
    redeem_stake_tx(ckb, staker_key.clone(), 10, 4)
        .await
        .unwrap();
    stake_smt_tx(ckb, kicker_key.clone(), stakers_key.clone(), 4).await;

    // New epoch
    run_metadata_tx(ckb, kicker_key.clone()).await;
    run_checkpoint_tx(ckb, kicker_key.clone(), stakers_key.clone(), 5).await;

    // Update withdraw cell:
    // (amount: 10, unlock epoch: 5), (amount: 10, unlock epoch: 6), (amount: 10,
    // unlock epoch: 7)
    redeem_stake_tx(ckb, staker_key.clone(), 30, 5)
        .await
        .unwrap();
    stake_smt_tx(ckb, kicker_key.clone(), stakers_key.clone(), 5).await;

    // New epoch
    run_checkpoint_tx(ckb, kicker_key.clone(), stakers_key.clone(), 6).await;
    run_checkpoint_tx(ckb, kicker_key.clone(), stakers_key.clone(), 7).await;
    run_checkpoint_tx(ckb, kicker_key.clone(), stakers_key.clone(), 8).await;

    // Withdraw all
    run_withdraw_tx(ckb, staker_key.clone(), 8).await;
}
