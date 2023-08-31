use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;
use storage::SmtManager;

use crate::config::types::PrivKeys;
use crate::helper::user::gen_users;
use crate::tx::*;

pub async fn run_stake_smt_case(ckb: &CkbRpcClient, smt: &SmtManager, priv_keys: PrivKeys) {
    if priv_keys.staker_privkeys.len() < 4 {
        panic!("At least 4 stakers are required");
    }

    let seeder_key = priv_keys.seeder_privkey.clone().into_h256().unwrap();
    let (stakers_key, _) = gen_users(priv_keys.staker_privkeys.clone());
    let kicker_key = stakers_key[0].clone();
    let stakers_key = vec![
        stakers_key[0].clone(),
        stakers_key[1].clone(),
        stakers_key[2].clone(),
        stakers_key[3].clone(),
    ];

    run_init_tx(ckb, seeder_key, stakers_key.clone(), 1).await;
    run_mint_tx(ckb, priv_keys.clone()).await;

    // staker1: 10
    first_stake_tx(ckb, stakers_key[0].clone(), 10).await;
    // staker2: 20
    first_stake_tx(ckb, stakers_key[1].clone(), 20).await;
    // staker3: 30
    first_stake_tx(ckb, stakers_key[2].clone(), 30).await;
    // staker4: 40
    first_stake_tx(ckb, stakers_key[3].clone(), 40).await;

    println!("\nThe removed staker1 is not in the stake smt");
    stake_smt_tx(ckb, smt, kicker_key.clone(), stakers_key.clone(), 0).await;
    // staker4: 40
    // staker3: 30
    // staker2: 20

    // staker2: -10
    redeem_stake_tx(ckb, stakers_key[1].clone(), 10, 0)
        .await
        .unwrap();

    // staker1: +5 -> +15
    add_stake_tx(ckb, stakers_key[0].clone(), 5, 0)
        .await
        .unwrap();

    println!("-------The remaining tests did not pass: 41-------");

    println!("The removed staker2 is in the stake smt");
    println!("There is a pending record of redeeming stake in the staker2's stake cell");
    stake_smt_tx(ckb, smt, kicker_key.clone(), stakers_key.clone(), 0).await;
    // staker4: 40
    // staker3: 30
    // staker1: 15

    // staker1: +5
    add_stake_tx(ckb, stakers_key[0].clone(), 5, 0)
        .await
        .unwrap();

    // new epoch
    run_checkpoint_tx(ckb, kicker_key.clone(), stakers_key.clone(), 1).await;

    // staker2: +35
    add_stake_tx(ckb, stakers_key[1].clone(), 35, 1)
        .await
        .unwrap();

    println!("\nThe removed staker1 is in the stake smt");
    println!("There is a expired record in the staker1's stake cell");
    stake_smt_tx(ckb, smt, kicker_key.clone(), stakers_key.clone(), 1).await;
    // staker4: 40
    // staker2: 35
    // staker3: 30
}
