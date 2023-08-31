use anyhow::Result;
use ckb_types::H256;
use common::types::tx_builder::{DelegateItem, EthAddress};
use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;
use storage::SmtManager;

use crate::config::types::PrivKeys;
use crate::helper::user::gen_users;
use crate::tx::*;

// The test did not pass
pub async fn run_metadata_case(ckb: &CkbRpcClient, smt: &SmtManager, priv_keys: PrivKeys) {
    if priv_keys.staker_privkeys.len() < 3 {
        panic!("At least 3 stakers are required");
    }

    if priv_keys.delegator_privkeys.is_empty() {
        panic!("At least one delegator is required");
    }

    let seeder_key = priv_keys.seeder_privkey.clone().into_h256().unwrap();
    let (stakers_key, stakers) = gen_users(priv_keys.staker_privkeys.clone());
    let (delegators_key, _) = gen_users(priv_keys.delegator_privkeys.clone());
    let kicker_key = stakers_key[0].clone();

    let stakers_key = vec![
        stakers_key[0].clone(),
        stakers_key[1].clone(),
        stakers_key[2].clone(),
    ];
    let stakers = vec![stakers[0].clone(), stakers[1].clone(), stakers[2].clone()];

    run_init_tx(ckb, seeder_key, stakers_key.clone(), 1).await;
    run_mint_tx(ckb, priv_keys.clone()).await;

    // staker1: 10
    first_stake_tx(ckb, stakers_key[0].clone(), 10).await;
    // staker2: 20
    first_stake_tx(ckb, stakers_key[1].clone(), 20).await;
    // staker3: 30
    first_stake_tx(ckb, stakers_key[2].clone(), 30).await;

    // delegator: (staker1, +10), (staker2, +10), (staker3, +10)
    first_delegates_tx(ckb, delegators_key[0].clone(), &stakers)
        .await
        .unwrap();

    stake_smt_tx(ckb, smt, kicker_key.clone(), stakers_key.clone(), 0).await;
    // staker3: 30
    // staker2: 20
    // staker1: 10

    delegate_smt_tx(
        ckb,
        smt,
        kicker_key.clone(),
        vec![delegators_key[0].clone()],
        0,
    )
    .await;
    // staker1: (delegator1, 10)
    // staker2: (delegator1, 10)
    // staker3: (delegator1, 10)

    // quorum = 1
    // (staker1, 10): (delegator1, 10)
    // (staker2, 20): (delegator1, 10)
    // (staker3, 30): (delegator1, 10)
    // Remove staker1 and staker2 from the stake smt
    // The delegator1's refunded amount should be added up to 20
    run_metadata_tx(ckb, smt, kicker_key.clone(), 0).await;
}

async fn first_delegates_tx(
    ckb: &CkbRpcClient,
    delegator_key: H256,
    stakers: &[EthAddress],
) -> Result<()> {
    println!("first delegate");

    let mut delegates = vec![];

    for staker in stakers.iter() {
        delegates.push(DelegateItem {
            staker:             staker.clone(),
            is_increase:        true,
            amount:             10,
            inauguration_epoch: 2,
        });
    }

    delegate_tx(ckb, delegator_key, delegates, 0, true).await
}
