use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;
use storage::SmtManager;

use crate::config::types::PrivKeys;
use crate::helper::user::gen_users;
use crate::tx::*;

pub async fn run_stake_case(ckb: &CkbRpcClient, smt: &SmtManager, priv_keys: PrivKeys) {
    if priv_keys.staker_privkeys.is_empty() {
        panic!("At least one stakers are required");
    }

    if priv_keys.delegator_privkeys.is_empty() {
        panic!("At least one delegator is required");
    }

    let seeder_key = priv_keys.seeder_privkey.clone().into_h256().unwrap();
    let (stakers_key, _) = gen_users(priv_keys.staker_privkeys.clone());
    let staker_key = stakers_key[0].clone();
    let kicker_key = stakers_key[0].clone();
    let stakers_key = vec![staker_key.clone()];

    run_init_tx(ckb, seeder_key, stakers_key.clone(), 10).await;
    run_mint_tx(ckb, priv_keys.clone()).await;

    // staker: +100
    first_stake_tx(ckb, stakers_key[0].clone(), 100).await;
    // wallet: 400, stake: 100, delta: +100

    // Stake too mutch
    assert!(add_stake_tx(ckb, stakers_key[0].clone(), 1000, 0)
        .await
        .is_err());

    // staker: -10
    println!(
        "\nWhen redeeming stake, there are pending records of adding stake with a larger amount"
    );
    redeem_stake_tx(ckb, staker_key.clone(), 10, 0)
        .await
        .unwrap();
    // wallet: 410, stake: 90, delta: +90

    // Clear all pending records
    stake_smt_tx(ckb, smt, kicker_key.clone(), vec![staker_key.clone()], 0).await;
    // wallet: 410, stake: 90, delta: 0

    // staker: -10
    redeem_stake_tx(ckb, staker_key.clone(), 10, 0)
        .await
        .unwrap();
    // wallet: 410, stake: 90, delta: -10

    // staker: -10
    println!("\nWhen redeeming stake, there are pending records of redeeming stake");
    redeem_stake_tx(ckb, staker_key.clone(), 10, 0)
        .await
        .unwrap();
    // wallet: 410, stake: 90, delta: -20

    // staker: +10
    println!(
        "\nWhen adding stake, there are pending records of redeeming stake with a larger amount"
    );
    add_stake_tx(ckb, staker_key.clone(), 10, 0).await.unwrap();
    // wallet: 410, stake: 90, delta: -10

    // staker: +15
    println!(
        "\nWhen adding stake, there are pending records of redeeming stake with a smaller amount"
    );
    add_stake_tx(ckb, staker_key.clone(), 15, 0).await.unwrap();
    // wallet: 405, stake: 95, delta: +5

    // staker: -15
    println!(
        "\nWhen redeeming stake, there are pending records of adding stake with a smaller amount"
    );
    redeem_stake_tx(ckb, staker_key.clone(), 15, 0)
        .await
        .unwrap();
    // wallet: 410, stake: 90, delta: -10

    // New epoch
    run_checkpoint_tx(ckb, kicker_key.clone(), stakers_key.clone(), 1).await;

    // staker: +10
    println!("\nWhen adding stake, there are expired records of redeeming stake");
    add_stake_tx(ckb, staker_key.clone(), 10, 1).await.unwrap();
    // wallet: 400, stake: 100, delta: +10

    // New epoch
    run_checkpoint_tx(ckb, kicker_key.clone(), stakers_key.clone(), 2).await;

    // staker: +15
    println!(
        "\nWhen adding stake, there are expired records of adding stake with a smaller amount"
    );
    add_stake_tx(ckb, staker_key.clone(), 15, 2).await.unwrap();
    // wallet: 395, stake: 105, delta: +15

    // New epoch
    run_checkpoint_tx(ckb, kicker_key.clone(), stakers_key.clone(), 3).await;

    // staker: +1
    println!("\nWhen adding stake, there are expired adding stake with a larger amount");
    add_stake_tx(ckb, staker_key.clone(), 1, 3).await.unwrap();
    // wallet: 409, stake: 91, delta: 1

    // staker: +10
    add_stake_tx(ckb, staker_key.clone(), 10, 3).await.unwrap();
    // wallet: 399, stake: 101, delta: +11

    // New epoch
    run_checkpoint_tx(ckb, kicker_key.clone(), stakers_key.clone(), 4).await;

    // staker: -15
    println!(
        "\nWhen redeeming stake, there are pending stale records of adding stake with a smaller amount"
    );
    redeem_stake_tx(ckb, staker_key.clone(), 15, 4)
        .await
        .unwrap();
    // wallet: 410, stake: 90, delta: -4

    // New epoch
    run_checkpoint_tx(ckb, kicker_key.clone(), stakers_key.clone(), 5).await;

    // staker: -1
    println!("\nWhen redeeming stake, there are pending records of redeeming stake");
    redeem_stake_tx(ckb, staker_key.clone(), 1, 5)
        .await
        .unwrap();
    // wallet: 410, stake: 90, delta: -1

    // staker: +11
    add_stake_tx(ckb, staker_key.clone(), 11, 5).await.unwrap();
    // wallet: 400, stake: 100, delta: +10

    // New epoch
    run_checkpoint_tx(ckb, kicker_key.clone(), stakers_key.clone(), 6).await;

    // staker: -1
    println!(
        "\nWhen redeeming stake, there are pending records of adding stake with a larger amount"
    );
    redeem_stake_tx(ckb, staker_key.clone(), 1, 6)
        .await
        .unwrap();
    // wallet: 410,  stake: 90, delta: 0
}
