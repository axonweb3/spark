use std::fs;
use std::path::Path;

use anyhow::Result;
use ckb_types::H256;
use common::types::tx_builder::{DelegateItem, EthAddress};
use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;
use tx_builder::ckb::helper::OmniEth;

use crate::config::types::PrivKeys;
use crate::tx::*;
use crate::ROCKSDB_PATH;

pub async fn run_delegate_case(ckb: &CkbRpcClient, priv_keys: PrivKeys) {
    if Path::new(ROCKSDB_PATH).exists() {
        fs::remove_dir_all(ROCKSDB_PATH).unwrap();
    }

    if priv_keys.staker_privkeys.len() < 2 {
        panic!("At least 2 stackers are required");
    }

    if priv_keys.delegator_privkeys.is_empty() {
        panic!("At least one delegator is required");
    }

    let stakers_key: Vec<H256> = priv_keys
        .staker_privkeys
        .clone()
        .into_iter()
        .map(|key| key.into_h256().unwrap())
        .collect();
    let stakers: Vec<EthAddress> = stakers_key
        .clone()
        .into_iter()
        .map(|key| OmniEth::new(key).address().unwrap())
        .collect();

    let delegator_key = priv_keys.delegator_privkeys[0].clone().into_h256().unwrap();

    let kicker_key = stakers_key[0].clone();

    for key in stakers_key.iter() {
        if key == &delegator_key {
            panic!("Stakers can't delegate themselves.");
        }
    }

    run_init_tx(ckb, priv_keys.clone()).await;
    run_mint_tx(ckb, priv_keys.clone()).await;

    first_stake_tx(ckb, stakers_key[0].clone()).await;
    first_stake_tx(ckb, stakers_key[1].clone()).await;

    first_delegate_tx(ckb, delegator_key.clone(), stakers[0].clone())
        .await
        .unwrap();
    // wallet: 400, delegate: 100, delta: +100

    // Delegate too mutch
    assert!(
        add_delegates(ckb, delegator_key.clone(), stakers.clone(), 1000)
            .await
            .is_err()
    );

    // The first staker exists in the delegate AT cell, while the second stacker
    // does not exist
    add_delegates(ckb, delegator_key.clone(), stakers.clone(), 10)
        .await
        .unwrap();
    // wallet: 390, delegate: 110, delta: +110

    // When redeeming delegation, there are pending records of adding delegation
    // with a larger amount
    redeem_delegate_tx(ckb, delegator_key.clone(), stakers[0].clone(), 10, 0)
        .await
        .unwrap();
    // wallet: 400, delegate: 100, delta: +100

    // Redeem from a staker who has never been delegated
    assert!(
        redeem_delegate_tx(ckb, delegator_key.clone(), stakers[1].clone(), 10, 0)
            .await
            .is_err()
    );

    // Clear all pending records
    delegate_smt_tx(ckb, kicker_key, vec![delegator_key.clone()]).await;
    // wallet: 400, delegate: 100, delta: 0

    // Redeem from a staker who has been delegated
    redeem_delegate_tx(ckb, delegator_key.clone(), stakers[0].clone(), 10, 0)
        .await
        .unwrap();
    // wallet: 400, delegate: 100, delta: -10

    // When redeeming delegation, there are pending records of redeeming delegation
    redeem_delegate_tx(ckb, delegator_key.clone(), stakers[0].clone(), 10, 0)
        .await
        .unwrap();
    // wallet: 400, delegate: 100, delta: -20

    // When adding delegation, there are pending records of redeeming delegation
    // with a larger amount
    add_delegate_tx(ckb, delegator_key.clone(), stakers[0].clone(), 10, 0)
        .await
        .unwrap();
    // wallet: 400, delegate: 100, delta: -10

    // When adding delegation, there are pending records of redeeming delegation
    // with a smaller amount
    add_delegate_tx(ckb, delegator_key.clone(), stakers[0].clone(), 15, 0)
        .await
        .unwrap();
    // wallet: 395, delegate: 105, delta: +5

    // When redeeming delegation, there are pending records of adding delegation
    // with a smaller amount
    redeem_delegate_tx(ckb, delegator_key.clone(), stakers[0].clone(), 15, 0)
        .await
        .unwrap();
    // wallet: 400, delegate: 100, delta: -10

    // New epoch
    run_checkpoint_tx(ckb, priv_keys.clone(), 1).await;

    // When adding delegation, there are expired records of redeeming delegation
    add_delegate_tx(ckb, delegator_key.clone(), stakers[0].clone(), 10, 1)
        .await
        .unwrap();
    // wallet: 390, delegate: 110, delta: +10

    // New epoch
    run_checkpoint_tx(ckb, priv_keys.clone(), 2).await;

    // When adding delegation, there are expired records of adding delegation with a
    // smaller amount
    add_delegate_tx(ckb, delegator_key.clone(), stakers[0].clone(), 15, 2)
        .await
        .unwrap();
    // wallet: 385, delegate: 115, delta: +5

    // New epoch
    run_checkpoint_tx(ckb, priv_keys.clone(), 3).await;

    // When adding delegation, there are expired adding delegation with a larger
    // amount
    add_delegate_tx(ckb, delegator_key.clone(), stakers[0].clone(), 1, 3)
        .await
        .unwrap();
    // wallet: 389, delegate: 111, delta: 0

    add_delegate_tx(ckb, delegator_key.clone(), stakers[0].clone(), 10, 3)
        .await
        .unwrap();
    // wallet: 379, delegate: 121, delta: +10

    // New epoch
    run_checkpoint_tx(ckb, priv_keys.clone(), 4).await;

    // When redeeming delegation, there are pending records of adding delegation
    // with a smaller amount
    redeem_delegate_tx(ckb, delegator_key.clone(), stakers[0].clone(), 15, 4)
        .await
        .unwrap();
    // wallet: 389, delegate: 111, delta: -5

    // New epoch
    run_checkpoint_tx(ckb, priv_keys.clone(), 4).await;

    // When redeeming delegation, there are pending records of redeeming delegation
    redeem_delegate_tx(ckb, delegator_key.clone(), stakers[0].clone(), 1, 4)
        .await
        .unwrap();
    // wallet: 389, delegate: 111, delta: -1

    add_delegate_tx(ckb, delegator_key.clone(), stakers[0].clone(), 11, 3)
        .await
        .unwrap();
    // wallet: 379, delegate: 121, delta: +10

    // New epoch
    run_checkpoint_tx(ckb, priv_keys.clone(), 5).await;

    // When redeeming delegation, there are pending records of adding delegation
    // with a larger amount
    redeem_delegate_tx(ckb, delegator_key.clone(), stakers[0].clone(), 1, 4)
        .await
        .unwrap();
    // wallet: 389,  delegate: 111, delta: 0
}

async fn add_delegates(
    ckb: &CkbRpcClient,
    delegator_key: H256,
    stakers_eth_addr: Vec<EthAddress>,
    amount: u128,
) -> Result<()> {
    println!("add delegate");

    let delegates = stakers_eth_addr
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
