use common::traits::tx_builder::IMintTxBuilder;
use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;
use tx_builder::ckb::helper::{OmniEth, Tx};
use tx_builder::ckb::mint::MintTxBuilder;

use crate::config::parse_type_ids;
use crate::config::types::PrivKeys;
use crate::TYPE_IDS_PATH;

pub async fn run_mint_tx(ckb: &CkbRpcClient, priv_keys: PrivKeys) {
    let type_ids = parse_type_ids(TYPE_IDS_PATH);

    let seeder_key = priv_keys.seeder_privkey.into_h256().unwrap();
    let omni_eth = OmniEth::new(seeder_key.clone());
    println!("seeder ckb addres: {}\n", omni_eth.ckb_address().unwrap());

    let mut stakers = vec![];
    for (i, staker_privkey) in priv_keys.staker_privkeys.into_iter().enumerate() {
        let privkey = staker_privkey.clone().into_h256().unwrap();
        let omni_eth = OmniEth::new(privkey);
        println!(
            "staker{} ckb addres: {}",
            i,
            omni_eth.ckb_address().unwrap()
        );
        stakers.push((omni_eth.address().unwrap(), 500));
    }

    let selection_type_id = type_ids.selection_type_id.into_h256().unwrap();
    let issue_type_id = type_ids.issue_type_id.into_h256().unwrap();

    let tx = MintTxBuilder::new(ckb, seeder_key, stakers, selection_type_id, issue_type_id)
        .build_tx()
        .await
        .unwrap();

    let mut tx = Tx::new(ckb, tx);

    match tx.send().await {
        Ok(tx_hash) => println!("mint tx hash: 0x{}", tx_hash),
        Err(e) => println!("{}", e),
    }

    println!("mint tx ready");
    tx.wait_until_committed(1000, 10).await.unwrap();
    println!("mint tx committed");
}
