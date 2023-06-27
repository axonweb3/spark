use ckb_sdk::unlock::ScriptSigner;
use common::traits::tx_builder::ICheckpointTxBuilder;
use common::types::tx_builder::{Checkpoint, CheckpointTypeIds, CkbNetwork};
use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;
use tx_builder::ckb::checkpoint::CheckpointTxBuilder;
use tx_builder::ckb::utils::omni::{omni_eth_ckb_address, omni_eth_signer};
use tx_builder::ckb::utils::tx::{gen_script_group, send_tx};

use crate::config::parse_file;
use crate::config::types::{PrivKeys, TypeIds as CTypeIds};
use crate::{PRIV_KEYS_PATH, TYPE_IDS_PATH};

pub async fn checkpoint_tx(ckb: CkbNetwork<CkbRpcClient>) {
    let priv_keys: PrivKeys = parse_file(PRIV_KEYS_PATH);
    let test_kicker_key = priv_keys.staker_privkeys[0].clone().into_h256().unwrap();
    println!(
        "kicker ckb addres: {}\n",
        omni_eth_ckb_address(&ckb.network_type, test_kicker_key.clone()).unwrap()
    );

    let type_ids: CTypeIds = parse_file(TYPE_IDS_PATH);
    let metadata_type_id = type_ids.metadata_type_id.into_h256().unwrap();
    let checkpoint_type_id = type_ids.checkpoint_type_id.into_h256().unwrap();

    let mut tx = CheckpointTxBuilder::new(
        test_kicker_key.clone(),
        ckb.clone(),
        CheckpointTypeIds {
            metadata_type_id,
            checkpoint_type_id,
        },
        2,
        Checkpoint {
            epoch: 1,
            period: 1,
            latest_block_height: 10,
            timestamp: 11111,
            ..Default::default()
        },
    )
    .await
    .build_tx()
    .await
    .unwrap();

    let signer = omni_eth_signer(test_kicker_key).unwrap();
    let script_groups = gen_script_group(&ckb.client, &tx).await.unwrap();
    let mut first_group = true;

    for group in script_groups.lock_groups.iter() {
        if !first_group {
            println!("sign; not checkpoint id: {:?}", group.1.input_indices);
            tx = signer.sign_tx(&tx, group.1).unwrap();
        } else {
            println!("not sign; checkpoint id: {:?}", group.1.input_indices);
        }
        first_group = false;
    }

    match send_tx(&ckb.client, &tx.data().into()).await {
        Ok(tx_hash) => println!("tx hash: 0x{}", tx_hash),
        Err(e) => println!("{}", e),
    }

    println!("\ntx: {}", tx);
}
