use common::traits::tx_builder::IMintTxBuilder;
use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;
use tx_builder::ckb::helper::{OmniEth, Tx};
use tx_builder::ckb::mint::MintTxBuilder;

use crate::config::parse_file;
use crate::config::types::{PrivKeys, TypeIds as CTypeIds};
use crate::{PRIV_KEYS_PATH, TYPE_IDS_PATH};

pub async fn mint_tx(ckb: &CkbRpcClient) {
    let priv_keys: PrivKeys = parse_file(PRIV_KEYS_PATH);
    let test_seeder_key = priv_keys.seeder_privkey.into_h256().unwrap();
    let omni_eth = OmniEth::new(test_seeder_key.clone());
    println!("seeder ckb addres: {}\n", omni_eth.ckb_address().unwrap());

    let mut stakers = vec![];
    for staker_privkey in priv_keys.staker_privkeys.into_iter() {
        let privkey = staker_privkey.clone().into_h256().unwrap();
        let omni_eth = OmniEth::new(privkey);
        println!("staker ckb addres: {}", omni_eth.ckb_address().unwrap());
        stakers.push((omni_eth.address().unwrap(), 200));
    }

    let type_ids: CTypeIds = parse_file(TYPE_IDS_PATH);
    let selection_type_id = type_ids.selection_type_id.into_h256().unwrap();
    let issue_type_id = type_ids.issue_type_id.into_h256().unwrap();

    let tx = MintTxBuilder::new(
        ckb,
        test_seeder_key,
        stakers,
        selection_type_id,
        issue_type_id,
    )
    .build_tx()
    .await
    .unwrap();

    let mut tx = Tx::new(ckb, tx);

    match tx.send().await {
        Ok(tx_hash) => println!("tx hash: 0x{}", tx_hash),
        Err(e) => println!("{}", e),
    }
}
