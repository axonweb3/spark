use common::traits::tx_builder::IInitTxBuilder;
use common::types::tx_builder::{Checkpoint, CkbNetwork, Metadata};
use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;
use tx_builder::ckb::init::InitTxBuilder;
use tx_builder::ckb::utils::omni::omni_eth_ckb_address;
use tx_builder::ckb::utils::tx::send_tx;

use crate::config::types::{PrivKeys, TypeIds as CTypeIds};
use crate::config::{parse_file, write_file};

use crate::{PRIV_KEYS_PATH, TYPE_IDS_PATH};

pub async fn init_tx(ckb: CkbNetwork<CkbRpcClient>) {
    let priv_keys: PrivKeys = parse_file(PRIV_KEYS_PATH);
    let test_seeder_key = priv_keys.seeder_privkey.into_h256().unwrap();
    println!(
        "seeder ckb addres: {}\n",
        omni_eth_ckb_address(&ckb.network_type, test_seeder_key.clone()).unwrap()
    );

    let (tx, type_id_args) = InitTxBuilder::new(
        ckb.clone(),
        test_seeder_key,
        10000,
        Checkpoint {
            epoch: 0,
            period: 0,
            ..Default::default()
        },
        Metadata {
            epoch_len: 2,
            period_len: 2,
            quorum: 2,
            ..Default::default()
        },
    )
    .build_tx()
    .await
    .unwrap();

    match send_tx(&ckb.client, &tx.data().into()).await {
        Ok(tx_hash) => println!("tx hash: 0x{}", tx_hash),
        Err(e) => println!("{}", e),
    }

    let type_ids: CTypeIds = type_id_args.into();
    write_file(TYPE_IDS_PATH, &type_ids);
}
