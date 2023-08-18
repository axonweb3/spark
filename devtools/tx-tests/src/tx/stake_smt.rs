use std::{path::PathBuf, vec};

use ckb_types::{prelude::Pack, H256};
use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;
use storage::SmtManager;

use common::traits::tx_builder::IStakeSmtTxBuilder;
use common::types::tx_builder::StakeSmtTypeIds;
use tx_builder::ckb::helper::{OmniEth, Stake, Tx, Xudt};
use tx_builder::ckb::stake_smt::StakeSmtTxBuilder;

use crate::config::parse_type_ids;
use crate::{MAX_TRY, ROCKSDB_PATH, TYPE_IDS_PATH};

pub async fn stake_smt_tx(ckb: &CkbRpcClient, kicker_key: H256, stakers_key: Vec<H256>) {
    let type_ids = parse_type_ids(TYPE_IDS_PATH);

    let omni_eth = OmniEth::new(kicker_key.clone());
    println!("kicker ckb addres: {}\n", omni_eth.ckb_address().unwrap());

    let metadata_type_id = type_ids.metadata_type_id.into_h256().unwrap();
    let checkpoint_type_id = type_ids.checkpoint_type_id.into_h256().unwrap();
    let stake_smt_type_id = type_ids.stake_smt_type_id.into_h256().unwrap();
    let xudt_owner = type_ids.xudt_owner.into_h256().unwrap();

    let mut stake_cells = vec![];
    for (i, staker_key) in stakers_key.into_iter().enumerate() {
        let omni_eth = OmniEth::new(staker_key.clone());
        println!(
            "staker{} ckb addres: {}\n",
            i,
            omni_eth.ckb_address().unwrap()
        );

        stake_cells.push(
            Stake::get_cell(
                ckb,
                Stake::lock(&metadata_type_id, &omni_eth.address().unwrap()),
                Xudt::type_(&xudt_owner.pack()),
            )
            .await
            .unwrap()
            .expect("stake AT cell not found"),
        );
    }

    let path = PathBuf::from(ROCKSDB_PATH);
    let smt = SmtManager::new(path);

    let (tx, _) = StakeSmtTxBuilder::new(
        ckb,
        kicker_key,
        0,
        StakeSmtTypeIds {
            metadata_type_id,
            checkpoint_type_id,
            stake_smt_type_id,
            xudt_owner,
        },
        stake_cells,
        smt,
    )
    .build_tx()
    .await
    .unwrap();

    let mut tx = Tx::new(ckb, tx);
    match tx.send().await {
        Ok(tx_hash) => println!("stake smt tx hash: 0x{}", tx_hash),
        Err(e) => println!("{}", e),
    }

    println!("stake smt tx ready");
    tx.wait_until_committed(1000, MAX_TRY).await.unwrap();
    println!("stake smt tx committed");
}
