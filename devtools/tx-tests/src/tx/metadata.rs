use std::path::PathBuf;

use ckb_types::H256;
use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;
use storage::SmtManager;

use common::traits::tx_builder::IMetadataTxBuilder;
use common::types::tx_builder::MetadataTypeIds;
use tx_builder::ckb::helper::{Checkpoint, Tx};
use tx_builder::ckb::metadata::MetadataSmtTxBuilder;

use crate::config::parse_type_ids;
use crate::{MAX_TRY, ROCKSDB_PATH, TYPE_IDS_PATH};

pub async fn run_metadata_tx(ckb: &CkbRpcClient, kicker_key: H256) {
    let type_ids = parse_type_ids(TYPE_IDS_PATH);

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
        kicker_key,
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
        Ok(tx_hash) => println!("metadata tx hash: 0x{}", tx_hash),
        Err(e) => println!("{}", e),
    }

    println!("metadata tx ready");
    tx.wait_until_committed(1000, MAX_TRY).await.unwrap();
    println!("metadata tx committed");
}
