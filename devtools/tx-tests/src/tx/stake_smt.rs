use std::{fs, path::PathBuf, vec};

use ckb_types::prelude::Pack;
use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;
use storage::SmtManager;

use common::traits::tx_builder::IStakeSmtTxBuilder;
use common::types::tx_builder::StakeSmtTypeIds;
use tx_builder::ckb::helper::{OmniEth, Stake, Tx, Xudt};
use tx_builder::ckb::stake_smt::StakeSmtTxBuilder;

use crate::config::parse_file;
use crate::config::types::{PrivKeys, TypeIds as CTypeIds};
use crate::{PRIV_KEYS_PATH, TYPE_IDS_PATH};

static ROCKSDB_PATH: &str = "./free-space/smt/stake";

pub async fn stake_smt_tx(ckb: &CkbRpcClient) {
    let priv_keys: PrivKeys = parse_file(PRIV_KEYS_PATH);
    let test_kicker_key = priv_keys.staker_privkeys[0].clone().into_h256().unwrap();
    let omni_eth = OmniEth::new(test_kicker_key.clone());
    println!("kicker ckb addres: {}\n", omni_eth.ckb_address().unwrap());

    let type_ids: CTypeIds = parse_file(TYPE_IDS_PATH);
    let metadata_type_id = type_ids.metadata_type_id.into_h256().unwrap();
    let checkpoint_type_id = type_ids.checkpoint_type_id.into_h256().unwrap();
    let stake_smt_type_id = type_ids.stake_smt_type_id.into_h256().unwrap();
    let xudt_owner = type_ids.xudt_owner.into_h256().unwrap();

    let stake_cell = Stake::get_cell(
        ckb,
        Stake::lock(&metadata_type_id, &omni_eth.address().unwrap()),
        Xudt::type_(&xudt_owner.pack()),
    )
    .await
    .unwrap()
    .unwrap();

    let path = PathBuf::from(ROCKSDB_PATH);
    if std::path::Path::new(ROCKSDB_PATH).exists() {
        fs::remove_dir_all(path.clone()).unwrap();
    }
    let smt = SmtManager::new(path);

    let (tx, _) = StakeSmtTxBuilder::new(
        ckb,
        test_kicker_key,
        0,
        StakeSmtTypeIds {
            metadata_type_id,
            checkpoint_type_id,
            stake_smt_type_id,
            xudt_owner,
        },
        10,
        vec![stake_cell],
        smt,
    )
    .build_tx()
    .await
    .unwrap();

    let mut tx = Tx::new(ckb, tx);
    match tx.send().await {
        Ok(tx_hash) => println!("tx hash: 0x{}", tx_hash),
        Err(e) => println!("{}", e),
    }

    println!("\ntx: {}", tx.inner());
}
