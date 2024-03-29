use std::path::PathBuf;

use anyhow::Result;
use ckb_types::{H160, H256};
use common::traits::smt::DelegateSmtStorage;
use common::types::smt::UserAmount;
use common::types::tx_builder::{DelegateItem, EthAddress};
use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;
use storage::SmtManager;

use crate::config::types::PrivKeys;
use crate::helper::smt::remove_smt;
use crate::helper::user::gen_users;
use crate::tx::*;
use crate::ROCKSDB_PATH;

pub async fn run_delegate_case(ckb: &CkbRpcClient, priv_keys: PrivKeys) {
    remove_smt();

    if priv_keys.staker_privkeys.len() < 3 {
        panic!("At least 3 stakers are required");
    }

    if priv_keys.delegator_privkeys.is_empty() {
        panic!("At least one delegator is required");
    }

    let seeder_key = priv_keys.seeder_privkey.clone().into_h256().unwrap();
    let (stakers_key, stakers) = gen_users(priv_keys.staker_privkeys.clone());
    let (delegators_key, delegators) = gen_users(priv_keys.delegator_privkeys.clone());
    let delegator_key = delegators_key[0].clone();
    let delegator = delegators[0].clone();
    let kicker_key = stakers_key[0].clone();
    let stakers_key = vec![stakers_key[0].clone(), stakers_key[1].clone()];
    let staker3 = stakers[2].clone();
    let staker = stakers[0].clone();
    let stakers = vec![stakers[0].clone(), stakers[1].clone()];
    let delegators_key = vec![delegator_key.clone()];

    for key in stakers_key.iter() {
        if key == &delegator_key {
            panic!("Stakers can't delegate themselves.");
        }
    }

    run_init_tx(ckb, seeder_key, stakers_key.clone(), 10).await;
    run_mint_tx(ckb, priv_keys.clone()).await;

    first_stake_tx(ckb, stakers_key[0].clone(), 100).await;
    first_stake_tx(ckb, stakers_key[1].clone(), 100).await;

    // delegator: (staker1, +100)
    first_delegate_tx(ckb, delegator_key.clone(), staker.clone())
        .await
        .unwrap();
    // wallet: 400, delegate: 100, delta: +100

    println!("\nDelegate too mutch");
    assert!(
        add_delegates(ckb, delegator_key.clone(), stakers.clone(), 1000)
            .await
            .is_err()
    );

    // delegator: (staker1, +10) (staker2, +10)
    println!(
        "\nThe first staker exists in the delegate AT cell, while the second staker does not exist"
    );
    add_delegates(ckb, delegator_key.clone(), stakers.clone(), 10)
        .await
        .unwrap();
    // wallet: 380, delegate: 120, delta: (staker1, +110), (staker2, +10)

    // delegator: (staker1, -10)
    println!("\nWhen redeeming delegation, there are pending records of adding delegation with a larger amount");
    redeem_delegate_tx(ckb, delegator_key.clone(), staker.clone(), 10, 0)
        .await
        .unwrap();
    // wallet: 390, delegate: 110, delta: (staker1, +100), (staker2, +10)

    println!("\nRedeem from a staker who has never been delegated");
    assert!(
        redeem_delegate_tx(ckb, delegator_key.clone(), staker3, 10, 0)
            .await
            .is_err()
    );

    println!("\nClear all pending records");
    delegate_smt_tx(ckb, kicker_key.clone(), delegators_key.clone(), 0).await;
    // wallet: 390, delegate: 110, delta: none

    // delegator: (staker1, -10)
    println!("\nRedeem from a staker who has been delegated");
    redeem_delegate_tx(ckb, delegator_key.clone(), staker.clone(), 10, 0)
        .await
        .unwrap();
    // wallet: 390, delegate: 110, delta: (staker1, -10)

    // delegator: (staker1, -10)
    println!("\nWhen redeeming delegation, there are pending records of redeeming delegation");
    redeem_delegate_tx(ckb, delegator_key.clone(), staker.clone(), 10, 0)
        .await
        .unwrap();
    // wallet: 390, delegate: 110, delta: (staker1, -20)

    // delegator: (staker1, +10)
    println!("\nWhen adding delegation, there are pending records of redeeming delegation with a larger amount");
    add_delegate_tx(ckb, delegator_key.clone(), staker.clone(), 10, 0)
        .await
        .unwrap();
    // wallet: 390, delegate: 110, delta: (staker1, -10)

    // delegator: (staker1, +15)
    println!("\nWhen adding delegation, there are pending records of redeeming delegation with a smaller amount");
    add_delegate_tx(ckb, delegator_key.clone(), staker.clone(), 15, 0)
        .await
        .unwrap();
    // wallet: 385, delegate: 115, delta: (staker1, +5)

    // delegator: (staker1, -15)
    println!("\nWhen redeeming delegation, there are pending records of adding delegation with a smaller amount");
    redeem_delegate_tx(ckb, delegator_key.clone(), staker.clone(), 15, 0)
        .await
        .unwrap();
    // wallet: 390, delegate: 110, delta: (staker1, -10)

    // New epoch
    run_checkpoint_tx(ckb, kicker_key.clone(), stakers_key.clone(), 1).await;

    // delegator: (staker1, +10)
    println!("\nWhen adding delegation, there are expired records of redeeming delegation");
    add_delegate_tx(ckb, delegator_key.clone(), staker.clone(), 10, 1)
        .await
        .unwrap();
    // wallet: 380, delegate: 120, delta: (staker1, +10)

    // New epoch
    run_checkpoint_tx(ckb, kicker_key.clone(), stakers_key.clone(), 2).await;

    // delegator: (staker1, +15)
    println!("\nWhen adding delegation, there are expired records of adding delegation with a smaller amount");
    add_delegate_tx(ckb, delegator_key.clone(), staker.clone(), 15, 2)
        .await
        .unwrap();
    // wallet: 375, delegate: 125, delta: (staker1, +15)

    // New epoch
    run_checkpoint_tx(ckb, kicker_key.clone(), stakers_key.clone(), 3).await;

    // delegator: (staker1, +1)
    println!("\nWhen adding delegation, there are expired adding delegation with a larger amount");
    add_delegate_tx(ckb, delegator_key.clone(), staker.clone(), 1, 3)
        .await
        .unwrap();
    // wallet: 389, delegate: 111, delta: (staker1, +1)

    // delegator: (staker1, +10)
    add_delegate_tx(ckb, delegator_key.clone(), staker.clone(), 10, 3)
        .await
        .unwrap();
    // wallet: 379, delegate: 121, delta: (staker1, +11)

    // New epoch
    run_checkpoint_tx(ckb, kicker_key.clone(), stakers_key.clone(), 4).await;

    // delegator: (staker1, -15)
    println!("\nWhen redeeming delegation, there are pending records of adding delegation with a smaller amount");
    mock_delegate_smt(6, &staker, &delegator).await;
    redeem_delegate_tx(ckb, delegator_key.clone(), staker.clone(), 15, 4)
        .await
        .unwrap();
    // wallet: 390, delegate: 110, delta: (staker1, -4)

    // New epoch
    run_checkpoint_tx(ckb, kicker_key.clone(), stakers_key.clone(), 5).await;

    // delegator: (staker1, -1)
    println!("\nWhen redeeming delegation, there are pending records of redeeming delegation");
    mock_delegate_smt(7, &staker, &delegator).await;
    redeem_delegate_tx(ckb, delegator_key.clone(), staker.clone(), 1, 5)
        .await
        .unwrap();
    // wallet: 390, delegate: 110, delta: (staker1, -1)

    // delegator: (staker1, +11)
    add_delegate_tx(ckb, delegator_key.clone(), staker.clone(), 11, 5)
        .await
        .unwrap();
    // wallet: 380, delegate: 120, delta: (staker1, +11)

    // New epoch
    run_checkpoint_tx(ckb, kicker_key.clone(), stakers_key.clone(), 6).await;

    // delegator: (staker1, -1)
    println!("\nWhen redeeming delegation, there are pending records of adding delegation with a larger amount");
    mock_delegate_smt(8, &staker, &delegator).await;
    redeem_delegate_tx(ckb, delegator_key.clone(), staker.clone(), 1, 6)
        .await
        .unwrap();
    // wallet: 390,  delegate: 110, delta: (staker1, 0)
}

async fn mock_delegate_smt(epoch: u64, staker: &H160, delegator: &H160) {
    let path = PathBuf::from(ROCKSDB_PATH);
    let smt = SmtManager::new(path);
    DelegateSmtStorage::insert(&smt, epoch, staker.0.into(), vec![UserAmount {
        user:        delegator.0.into(),
        amount:      100,
        is_increase: true,
    }])
    .await
    .unwrap();
}

async fn add_delegates(
    ckb: &CkbRpcClient,
    delegator_key: H256,
    stakers: Vec<EthAddress>,
    amount: u128,
) -> Result<()> {
    println!("add delegation");

    let delegates = stakers
        .into_iter()
        .map(|staker| DelegateItem {
            staker,
            is_increase: true,
            amount,
            inauguration_epoch: 2,
        })
        .collect();

    delegate_tx(ckb, delegator_key, delegates, 0, false).await
}
