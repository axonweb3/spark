use common::traits::tx_builder::IMintTxBuilder;
use common::types::tx_builder::CkbNetwork;
use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;
use tx_builder::ckb::mint::MintTxBuilder;
use tx_builder::ckb::utils::omni::{omni_eth_address, omni_eth_ckb_address};
use tx_builder::ckb::utils::tx::send_tx;

use crate::config::parse_file;
use crate::config::types::{PrivKeys, TypeIds as CTypeIds};
use crate::{PRIV_KEYS_PATH, TYPE_IDS_PATH};

pub async fn mint_tx(ckb: CkbNetwork<CkbRpcClient>) {
    let priv_keys: PrivKeys = parse_file(PRIV_KEYS_PATH);
    let test_seeder_key = priv_keys.seeder_privkey.into_h256().unwrap();
    println!(
        "seeder ckb addres: {}",
        omni_eth_ckb_address(&ckb.network_type, test_seeder_key.clone()).unwrap()
    );

    let mut stakers = vec![];
    for staker_privkey in priv_keys.staker_privkeys.into_iter() {
        let privkey = staker_privkey.clone().into_h256().unwrap();
        println!(
            "staker ckb addres: {}",
            omni_eth_ckb_address(&ckb.network_type, privkey.clone()).unwrap()
        );
        stakers.push((omni_eth_address(privkey.clone()).unwrap(), 200));
    }

    let type_ids: CTypeIds = parse_file(TYPE_IDS_PATH);
    let selection_type_id = type_ids.selection_type_id.into_h256().unwrap();
    let issue_type_id = type_ids.issue_type_id.into_h256().unwrap();

    let tx = MintTxBuilder::new(
        ckb.clone(),
        test_seeder_key,
        stakers,
        selection_type_id,
        issue_type_id,
    )
    .build_tx()
    .await
    .unwrap();

    match send_tx(&ckb.client, &tx.data().into()).await {
        Ok(tx_hash) => println!("tx hash: 0x{}", tx_hash),
        Err(e) => println!("{}", e),
    }
}
