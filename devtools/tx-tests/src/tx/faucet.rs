use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;
use tx_builder::ckb::faucet::FaucetTxBuilder;
use tx_builder::ckb::helper::{OmniEth, Sighash, Tx};

use crate::config::parse_file;
use crate::config::types::PrivKeys;
use crate::PRIV_KEYS_PATH;

pub async fn faucet_tx(ckb: &CkbRpcClient) {
    let priv_keys: PrivKeys = parse_file(PRIV_KEYS_PATH);
    let test_seeder_key = priv_keys.seeder_privkey.into_h256().unwrap();

    let omni_eth = OmniEth::new(test_seeder_key.clone());
    println!(
        "seeder omni eth ckb addres: {}\n",
        omni_eth.ckb_address().unwrap()
    );

    let sig_hash = Sighash::new(test_seeder_key.clone());
    println!(
        "seeder secp256k1 ckb addres: {}\n",
        sig_hash.address().unwrap()
    );

    let tx = FaucetTxBuilder::new(ckb, test_seeder_key, 1000000)
        .build_tx()
        .await
        .unwrap();

    let mut tx = Tx::new(ckb, tx);

    match tx.send().await {
        Ok(tx_hash) => println!("tx hash: 0x{}", tx_hash),
        Err(e) => println!("{}", e),
    }
}
