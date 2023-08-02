use std::fs;
use std::path::Path;

use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;
use tx_builder::ckb::helper::OmniEth;

use crate::config::types::PrivKeys;
use crate::tx::*;
use crate::ROCKSDB_PATH;

// There is only one staker or delegator.
pub async fn run_case1(ckb: &CkbRpcClient, priv_keys: PrivKeys) {
    if Path::new(ROCKSDB_PATH).exists() {
        fs::remove_dir_all(ROCKSDB_PATH).unwrap();
    }

    let kicker_key = priv_keys.staker_privkeys[0].clone().into_h256().unwrap();
    let staker_key = priv_keys.staker_privkeys[0].clone().into_h256().unwrap();
    let delegator_key = priv_keys.delegator_privkeys[0].clone().into_h256().unwrap();
    let staker_eth_addr = OmniEth::new(staker_key.clone()).address().unwrap();

    run_init_tx(ckb, priv_keys.clone()).await;
    run_mint_tx(ckb, priv_keys.clone()).await;

    first_stake_tx(ckb, staker_key.clone()).await;
    add_stake_tx(ckb, staker_key.clone(), 2).await;
    run_stake_smt_tx(ckb, kicker_key.clone()).await;

    reedem_stake_tx(ckb, staker_key.clone()).await;
    run_stake_smt_tx(ckb, kicker_key.clone()).await;

    first_delegate_tx(ckb, delegator_key.clone(), staker_eth_addr.clone()).await;
    add_delegate_tx(ckb, delegator_key.clone(), staker_eth_addr.clone()).await;
    run_delegate_smt_tx(ckb, kicker_key.clone()).await;

    reedem_delegate_tx(ckb, delegator_key, staker_eth_addr).await;
    run_delegate_smt_tx(ckb, kicker_key.clone()).await;

    run_metadata_tx(ckb, kicker_key.clone()).await;

    run_checkpoint_tx(ckb, priv_keys.clone(), 1).await;
    run_metadata_tx(ckb, kicker_key.clone()).await;

    run_checkpoint_tx(ckb, priv_keys.clone(), 2).await;
    run_metadata_tx(ckb, kicker_key.clone()).await;

    run_reward_tx(ckb, staker_key.clone()).await;

    run_withdraw_tx(ckb, staker_key.clone()).await;
}
