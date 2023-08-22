use std::fs;
use std::path::Path;

use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;
use tx_builder::ckb::helper::OmniEth;

use crate::config::types::PrivKeys;
use crate::tx::*;
use crate::ROCKSDB_PATH;

// There is only one staker or delegator.
pub async fn run_all_tx(ckb: &CkbRpcClient, priv_keys: PrivKeys) {
    if Path::new(ROCKSDB_PATH).exists() {
        fs::remove_dir_all(ROCKSDB_PATH).unwrap();
    }

    let seeder_key = priv_keys.seeder_privkey.clone().into_h256().unwrap();
    let kicker_key = priv_keys.staker_privkeys[0].clone().into_h256().unwrap();
    let staker_key = priv_keys.staker_privkeys[0].clone().into_h256().unwrap();
    let delegator_key = priv_keys.delegator_privkeys[0].clone().into_h256().unwrap();
    let staker = OmniEth::new(staker_key.clone()).address().unwrap();
    let stakers_key = vec![staker_key.clone()];

    if staker_key == delegator_key {
        panic!("Stakers can't delegate themselves.");
    }

    run_init_tx(ckb, seeder_key, stakers_key.clone(), 10).await;
    run_mint_tx(ckb, priv_keys.clone()).await;

    first_stake_tx(ckb, staker_key.clone(), 100).await;
    add_stake_tx(ckb, staker_key.clone(), 10, 0).await.unwrap();
    stake_smt_tx(ckb, kicker_key.clone(), vec![staker_key.clone()], 0).await;

    redeem_stake_tx(ckb, staker_key.clone(), 10, 0)
        .await
        .unwrap();
    stake_smt_tx(ckb, kicker_key.clone(), vec![staker_key.clone()], 0).await;

    first_delegate_tx(ckb, delegator_key.clone(), staker.clone())
        .await
        .unwrap();
    add_delegate_tx(ckb, delegator_key.clone(), staker.clone(), 10, 0)
        .await
        .unwrap();
    delegate_smt_tx(ckb, kicker_key.clone(), vec![delegator_key.clone()], 0).await;

    redeem_delegate_tx(ckb, delegator_key.clone(), staker, 10, 0)
        .await
        .unwrap();
    delegate_smt_tx(ckb, kicker_key.clone(), vec![delegator_key.clone()], 0).await;

    run_metadata_tx(ckb, kicker_key.clone()).await;

    run_checkpoint_tx(ckb, kicker_key.clone(), stakers_key.clone(), 1).await;
    run_metadata_tx(ckb, kicker_key.clone()).await;

    run_checkpoint_tx(ckb, kicker_key.clone(), stakers_key.clone(), 2).await;
    run_metadata_tx(ckb, kicker_key.clone()).await;

    run_reward_tx(ckb, staker_key.clone(), 4).await.unwrap();

    run_withdraw_tx(ckb, staker_key.clone(), 4).await;
}
