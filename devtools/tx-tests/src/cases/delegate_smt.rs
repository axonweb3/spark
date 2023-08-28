use std::fs;
use std::path::Path;

use anyhow::Result;
use ckb_types::H256;
use common::types::tx_builder::{DelegateItem, EthAddress};
use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;

use crate::config::types::PrivKeys;
use crate::helper::user::gen_users;
use crate::tx::*;
use crate::ROCKSDB_PATH;

pub async fn run_delegate_smt_case(ckb: &CkbRpcClient, priv_keys: PrivKeys) {
    if Path::new(ROCKSDB_PATH).exists() {
        fs::remove_dir_all(ROCKSDB_PATH).unwrap();
    }

    if priv_keys.staker_privkeys.len() < 2 {
        panic!("At least 2 stackers are required");
    }

    if priv_keys.delegator_privkeys.len() < 2 {
        panic!("At least 2 delegators are required");
    }

    let seeder_key = priv_keys.seeder_privkey.clone().into_h256().unwrap();
    let (stakers_key, stakers) = gen_users(priv_keys.staker_privkeys.clone());
    let (delegators_key, _) = gen_users(priv_keys.delegator_privkeys.clone());
    let kicker_key = stakers_key[0].clone();
    let stakers_key = vec![stakers_key[0].clone(), stakers_key[1].clone()];
    let delegators_key = vec![delegators_key[0].clone(), delegators_key[1].clone()];

    run_init_tx(ckb, seeder_key, stakers_key.clone(), 10).await;
    run_mint_tx(ckb, priv_keys.clone()).await;

    first_stake_tx(ckb, stakers_key[0].clone(), 100).await;
    first_stake_tx(ckb, stakers_key[1].clone(), 100).await;

    // delegator1: (staker1, +10), (staker2, +10)
    first_delegate_tx(ckb, delegators_key[0].clone(), &stakers, 10)
        .await
        .unwrap();

    // delegator2: (staker1, +20), (staker2, +20)
    first_delegate_tx(ckb, delegators_key[1].clone(), &stakers, 20)
        .await
        .unwrap();

    println!("\nThe removed delegator1 is not in the staker1's delegate smt");
    println!("The removed delegator1 is not in the staker2's delegate smt");
    delegate_smt_tx(ckb, kicker_key.clone(), delegators_key.clone(), 0).await;
    // staker1: (delegator2, 20)
    // staker2: (delegator2, 20)

    // delegator1: (staker1, +30) (staker2, +30)
    add_delegates_tx(ckb, delegators_key[0].clone(), &stakers)
        .await
        .unwrap();

    println!("-------The remaining tests did not pass: 117-------");

    println!("\nThe removed delegator2 is in the staker1's delegate smt");
    println!("The removed delegator2 is in the staker2's delegate smt");
    println!("The delegator2's refunded amount should be added up to 40");
    delegate_smt_tx(ckb, kicker_key.clone(), delegators_key.clone(), 0).await;
    // staker1: (delegator1, 30)
    // staker2: (delegator1, 30)

    // delegator1: (staker1, -10)
    redeem_delegate_tx(ckb, delegators_key[0].clone(), stakers[0].clone(), 10, 0)
        .await
        .unwrap();

    // delegator2: (staker1, +25)
    add_delegate_tx(ckb, delegators_key[1].clone(), stakers[0].clone(), 25, 0)
        .await
        .unwrap();

    println!("\nThe removed delegator1 is in the staker1's delegate smt");
    println!("There is a pending record of redeeming delegation in the delegator1's delegate cell");
    delegate_smt_tx(ckb, kicker_key.clone(), delegators_key.clone(), 0).await;
    // staker1: (delegator2, 25)
    // staker2: (delegator1, 30)

    // delegator2: (staker1, +5)
    add_delegate_tx(ckb, delegators_key[1].clone(), stakers[0].clone(), 5, 0)
        .await
        .unwrap();

    // new epoch
    run_checkpoint_tx(ckb, kicker_key.clone(), stakers_key.clone(), 1).await;

    // delegator1: (staker1, +40)
    add_delegate_tx(ckb, delegators_key[0].clone(), stakers[0].clone(), 40, 1)
        .await
        .unwrap();

    println!("\nThe removed delegator2 is in the staker1's delegate smt");
    println!("There is a expired record in the delegator2's delegate cell");
    delegate_smt_tx(ckb, kicker_key.clone(), delegators_key.clone(), 0).await;
    // staker1: (delegator2, 40)
    // staker2: (delegator1, 30)
}

async fn first_delegate_tx(
    ckb: &CkbRpcClient,
    delegator_key: H256,
    stakers: &Vec<EthAddress>,
    amount: u128,
) -> Result<()> {
    println!("first delegate");
    assert!(stakers.len() >= 2);

    delegate_tx(
        ckb,
        delegator_key,
        vec![
            DelegateItem {
                staker: stakers[0].clone(),
                is_increase: true,
                amount,
                inauguration_epoch: 2,
            },
            DelegateItem {
                staker: stakers[1].clone(),
                is_increase: true,
                amount,
                inauguration_epoch: 2,
            },
        ],
        0,
        true,
    )
    .await
}

async fn add_delegates_tx(
    ckb: &CkbRpcClient,
    delegator_key: H256,
    stakers_eth_addr: &Vec<EthAddress>,
) -> Result<()> {
    println!("add delegation");
    assert!(stakers_eth_addr.len() >= 2);

    delegate_tx(
        ckb,
        delegator_key,
        vec![
            DelegateItem {
                staker:             stakers_eth_addr[0].clone(),
                is_increase:        true,
                amount:             30,
                inauguration_epoch: 2,
            },
            DelegateItem {
                staker:             stakers_eth_addr[1].clone(),
                is_increase:        true,
                amount:             30,
                inauguration_epoch: 2,
            },
        ],
        0,
        false,
    )
    .await
}
