use std::collections::HashMap;

use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;
use tx_builder::ckb::helper::{OmniEth, Tx};
use tx_builder::ckb::mint::MintTxBuilder;

use crate::config::parse_type_ids;
use crate::config::types::PrivKeys;
use crate::{MAX_TRY, TYPE_IDS_PATH};

pub async fn run_mint_tx(ckb: &CkbRpcClient, priv_keys: PrivKeys) {
    let type_ids = parse_type_ids(TYPE_IDS_PATH);

    let seeder_key = priv_keys.seeder_privkey.into_h256().unwrap();

    let mut users = HashMap::new();

    for staker_privkey in priv_keys.staker_privkeys {
        let privkey = staker_privkey.clone().into_h256().unwrap();
        let omni_eth = OmniEth::new(privkey);
        users.insert(omni_eth.address().unwrap(), 500);
    }

    for delegator_privkey in priv_keys.delegator_privkeys {
        let privkey = delegator_privkey.clone().into_h256().unwrap();
        let omni_eth = OmniEth::new(privkey);
        users.insert(omni_eth.address().unwrap(), 500);
    }

    let users = users.into_iter().collect();

    let selection_type_id = type_ids.selection_type_id.into_h256().unwrap();
    let issue_type_id = type_ids.issue_type_id.into_h256().unwrap();

    let tx = MintTxBuilder::new(ckb, seeder_key, users, selection_type_id, issue_type_id)
        .build_tx()
        .await
        .unwrap();

    let mut tx = Tx::new(ckb, tx);

    match tx.send().await {
        Ok(tx_hash) => println!("mint tx hash: 0x{}", tx_hash),
        Err(e) => println!("{}", e),
    }

    println!("mint tx ready");
    tx.wait_until_committed(1000, MAX_TRY).await.unwrap();
    println!("mint tx committed");
}
