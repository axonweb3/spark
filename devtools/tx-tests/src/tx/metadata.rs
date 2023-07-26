use std::path::PathBuf;

use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;
use storage::SmtManager;

use common::traits::tx_builder::IMetadataTxBuilder;
use common::types::tx_builder::MetadataTypeIds;
use tx_builder::ckb::helper::{Checkpoint, OmniEth, Tx};
use tx_builder::ckb::metadata::MetadataSmtTxBuilder;

use crate::config::parse_file;
use crate::config::types::{PrivKeys, TypeIds as CTypeIds};
use crate::{PRIV_KEYS_PATH, TYPE_IDS_PATH};

static ROCKSDB_PATH: &str = "./free-space/smt";

pub async fn metadata_tx(ckb: &CkbRpcClient) {
    let priv_keys: PrivKeys = parse_file(PRIV_KEYS_PATH);
    let test_kicker_key = priv_keys.staker_privkeys[0].clone().into_h256().unwrap();
    let omni_eth = OmniEth::new(test_kicker_key.clone());
    println!("kicker ckb addres: {}\n", omni_eth.ckb_address().unwrap());

    let type_ids: CTypeIds = parse_file(TYPE_IDS_PATH);
    let metadata_type_id = type_ids.metadata_type_id.into_h256().unwrap();
    let checkpoint_type_id = type_ids.checkpoint_type_id.into_h256().unwrap();
    let stake_smt_type_id = type_ids.stake_smt_type_id.into_h256().unwrap();
    let delegate_smt_type_id = type_ids.delegate_smt_type_id.into_h256().unwrap();
    let xudt_owner = type_ids.xudt_owner.into_h256().unwrap();

    let checkpoint_cell = Checkpoint::get_cell(ckb, Checkpoint::type_(&checkpoint_type_id))
        .await
        .unwrap();

    let path = PathBuf::from(ROCKSDB_PATH);
    let smt = SmtManager::new(path);
    // disable load context from file
    let tmp_dir = tempfile::tempdir().unwrap();

    let tx = MetadataSmtTxBuilder::new(
        ckb,
        test_kicker_key,
        MetadataTypeIds {
            metadata_type_id,
            stake_smt_type_id,
            delegate_smt_type_id,
            xudt_owner,
        },
        checkpoint_cell,
        smt,
        tmp_dir.path().to_path_buf(),
    )
    .await
    .build_tx()
    .await
    .unwrap();

    let mut tx = Tx::new(ckb, tx);
    match tx.send().await {
        Ok(tx_hash) => println!("tx hash: 0x{}", tx_hash),
        Err(e) => println!("{}", e),
    }

    // println!("\ntx: {}", tx.inner());
}
