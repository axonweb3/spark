use anyhow::Result;
use ckb_types::H256;
use common::types::tx_builder::{DelegateItem, EthAddress};
use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;

use crate::config::types::PrivKeys;
use crate::helper::misc::remove_smt;
use crate::helper::user::gen_users;
use crate::tx::*;

pub async fn run_delegate_smt_case(ckb: &CkbRpcClient, priv_keys: PrivKeys) {
    remove_smt();

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

    // delegator1: (staker1, +10), (staker2, +10), 20
    first_delegate_tx(ckb, delegators_key[0].clone(), &stakers, 10)
        .await
        .unwrap();

    // delegator2: (staker1, +20), (staker2, +20), 40
    first_delegate_tx(ckb, delegators_key[1].clone(), &stakers, 20)
        .await
        .unwrap();

    println!("\nThe removed delegator1 is not in the staker1's delegate smt");
    println!("The removed delegator1 is not in the staker2's delegate smt");
    delegate_smt_tx(ckb, kicker_key.clone(), delegators_key.clone(), 0).await;
    // delegate-staker1 smt: (delegator2, 20)
    // delegate-staker2 smt: (delegator2, 20)

    // delegator1: (staker1, +30) (staker2, +30), 20 + 60 = 80
    add_delegates_tx(ckb, delegators_key[0].clone(), &stakers)
        .await
        .unwrap();

    println!("\nThe removed delegator2 is in the staker1's delegate smt");
    println!("The removed delegator2 is in the staker2's delegate smt");
    println!("The delegator2's refunded amount should be added up to 40");
    delegate_smt_tx(ckb, kicker_key.clone(), delegators_key.clone(), 0).await;
    // delegate-staker1 smt: (delegator1, 30)
    // delegate-staker2 smt: (delegator1, 30)
    // delegator1 cell: 80
    // delegator2 cell: 40 - 40 = 0

    // delegator1: (staker1, -10), 80
    redeem_delegate_tx(ckb, delegators_key[0].clone(), stakers[0].clone(), 10, 0)
        .await
        .unwrap();

    // delegator2: (staker1, +25), 25
    add_delegate_tx(ckb, delegators_key[1].clone(), stakers[0].clone(), 25, 0)
        .await
        .unwrap();

    println!("\nThe removed delegator1 is in the staker1's delegate smt");
    println!("There is a pending record of redeeming delegation in the delegator1's delegate cell");
    delegate_smt_tx(ckb, kicker_key.clone(), delegators_key.clone(), 0).await;
    // delegate-staker1 smt: (delegator2, 25)
    // delegate-staker2 smt: (delegator1, 30)
    // delegator1 cell: 80 - 30 = 50
    // delegator2 cell: 25

    // delegator2: (staker1, +5), 30
    add_delegate_tx(ckb, delegators_key[1].clone(), stakers[0].clone(), 5, 0)
        .await
        .unwrap();

    // new epoch
    run_checkpoint_tx(ckb, kicker_key.clone(), stakers_key.clone(), 1).await;

    // delegator1: (staker1, +40), 90
    add_delegate_tx(ckb, delegators_key[0].clone(), stakers[0].clone(), 40, 1)
        .await
        .unwrap();

    println!("\nThe removed delegator2 is in the staker1's delegate smt");
    println!("There is a expired record in the delegator2's delegate cell");
    delegate_smt_tx(ckb, kicker_key.clone(), delegators_key.clone(), 1).await;
    // delegate-staker1 smt: (delegator1, 40)
    // delegate-staker2 smt: (delegator1, 30)
    // delegator1 cell: 90
    // delegator2 cell: 25
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
