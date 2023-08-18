use std::fs;
use std::path::Path;

use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;

use crate::config::types::PrivKeys;
use crate::helper::user::get_users;
use crate::tx::*;
use crate::ROCKSDB_PATH;

pub async fn run_stake_smt_case(ckb: &CkbRpcClient, priv_keys: PrivKeys) {
    if Path::new(ROCKSDB_PATH).exists() {
        fs::remove_dir_all(ROCKSDB_PATH).unwrap();
    }

    if priv_keys.staker_privkeys.len() < 4 {
        panic!("At least 4 stakers are required");
    }

    let (stakers_key, _) = get_users(priv_keys.staker_privkeys.clone());
    let kicker_key = stakers_key[0].clone();

    run_init_tx(ckb, priv_keys.clone(), 1).await;
    run_mint_tx(ckb, priv_keys.clone()).await;

    // staker1: 10
    first_stake_tx(ckb, stakers_key[0].clone(), 10).await;
    // staker2: 20
    first_stake_tx(ckb, stakers_key[1].clone(), 20).await;
    // staker3: 30
    first_stake_tx(ckb, stakers_key[2].clone(), 30).await;
    // staker4: 40
    first_stake_tx(ckb, stakers_key[3].clone(), 40).await;

    // The removed staker1 is not in the stake smt
    stake_smt_tx(ckb, kicker_key.clone(), stakers_key.clone()).await;
    // staker4: 40
    // staker3: 30
    // staker2: 20

    // staker2: -10
    redeem_stake_tx(ckb, stakers_key[1].clone(), 10, 0)
        .await
        .unwrap();

    // staker1: +15
    add_stake_tx(ckb, stakers_key[0].clone(), 15, 0)
        .await
        .unwrap();

    // The removed staker2 is in the stake smt
    // There is a pending record of redeeming stake in the staker2's stake cell
    delegate_smt_tx(ckb, kicker_key.clone(), stakers_key.clone()).await;
    // staker4: 40
    // staker3: 30
    // staker1: 15

    // staker1: +5
    add_stake_tx(ckb, stakers_key[0].clone(), 5, 0)
        .await
        .unwrap();

    // new epoch
    run_checkpoint_tx(ckb, priv_keys.clone(), 1).await;

    // staker2: +35
    add_stake_tx(ckb, stakers_key[1].clone(), 35, 1)
        .await
        .unwrap();

    // The removed staker1 is in the stake smt
    // There is a expired record in the staker1's stake cell
    delegate_smt_tx(ckb, kicker_key.clone(), stakers_key.clone()).await;
    // staker4: 40
    // staker2: 35
    // staker3: 30
}
