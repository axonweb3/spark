use std::fs;
use std::path::Path;

use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;

use crate::config::types::PrivKeys;
use crate::helper::user::get_users;
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

    let (stakers_key, stakers) = get_users(priv_keys.staker_privkeys.clone());
    let (delegators_key, _) = get_users(priv_keys.staker_privkeys.clone());
    let kicker_key = stakers_key[0].clone();

    run_init_tx(ckb, priv_keys.clone(), 1).await;
    run_mint_tx(ckb, priv_keys.clone()).await;

    first_stake_tx(ckb, stakers_key[0].clone(), 100).await;
    stake_smt_tx(ckb, kicker_key.clone(), vec![stakers_key[0].clone()]).await;

    first_delegate_tx(ckb, delegators_key[0].clone(), stakers[0].clone())
        .await
        .unwrap();
    delegate_smt_tx(ckb, kicker_key.clone(), vec![delegators_key[0].clone()]).await;

    run_metadata_tx(ckb, kicker_key.clone()).await;

    run_checkpoint_tx(ckb, priv_keys.clone(), 1).await;
    run_metadata_tx(ckb, kicker_key.clone()).await;

    run_checkpoint_tx(ckb, priv_keys.clone(), 2).await;
    run_metadata_tx(ckb, kicker_key.clone()).await;

    assert!(run_reward_tx(ckb, stakers_key[0].clone(), 3).await.is_err());

    // Validator
    run_reward_tx(ckb, stakers_key[0].clone(), 4).await.unwrap();

    // Claim the reward again
    assert!(run_reward_tx(ckb, stakers_key[0].clone(), 4).await.is_err());

    // Delegator
    run_reward_tx(ckb, delegators_key[0].clone(), 4)
        .await
        .unwrap();

    // Neither a validator nor delegator
    run_reward_tx(ckb, delegators_key[1].clone(), 4)
        .await
        .unwrap();
}
