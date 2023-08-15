use std::collections::HashMap;

use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;
use tx_builder::ckb::faucet::FaucetTxBuilder;
use tx_builder::ckb::helper::{OmniEth, Sighash, Tx};

use crate::config::types::PrivKeys;
use crate::MAX_TRY;

pub async fn run_faucet_tx(ckb: &CkbRpcClient, priv_keys: PrivKeys) {
    let seeder_key = priv_keys.seeder_privkey.into_h256().unwrap();

    let omni_eth = OmniEth::new(seeder_key.clone());
    println!(
        "seeder omni eth ckb addres: {}\n",
        omni_eth.ckb_address().unwrap()
    );

    let mut users = HashMap::new();

    for (i, staker_privkey) in priv_keys.staker_privkeys.into_iter().enumerate() {
        let privkey = staker_privkey.clone().into_h256().unwrap();
        let omni_eth = OmniEth::new(privkey);
        println!(
            "staker{} ckb addres: {}",
            i,
            omni_eth.ckb_address().unwrap(),
        );
        users.insert(omni_eth.address().unwrap(), 10000);
    }

    for (i, delegator_privkey) in priv_keys.delegator_privkeys.into_iter().enumerate() {
        let privkey = delegator_privkey.clone().into_h256().unwrap();
        let omni_eth = OmniEth::new(privkey);
        println!(
            "staker{} ckb addres: {}",
            i,
            omni_eth.ckb_address().unwrap(),
        );
        users.insert(omni_eth.address().unwrap(), 10000);
    }

    let users = users.into_iter().collect();

    let sig_hash = Sighash::new(seeder_key.clone());
    println!(
        "seeder secp256k1 ckb addres: {}\n",
        sig_hash.address().unwrap()
    );

    let tx = FaucetTxBuilder::new(ckb, seeder_key, users)
        .build_tx()
        .await
        .unwrap();

    let mut tx = Tx::new(ckb, tx);

    match tx.send().await {
        Ok(tx_hash) => println!("faucet tx hash: 0x{}", tx_hash),
        Err(e) => println!("{}", e),
    }

    println!("faucet tx ready");
    tx.wait_until_committed(1000, MAX_TRY).await.unwrap();
    println!("faucet tx committed");
}
