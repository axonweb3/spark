use std::{path::PathBuf, vec};

use ckb_types::{prelude::Pack, H256};
use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;
use storage::SmtManager;

use common::traits::tx_builder::IDelegateSmtTxBuilder;
use common::types::tx_builder::DelegateSmtTypeIds;
use tx_builder::ckb::delegate_smt::DelegateSmtTxBuilder;
use tx_builder::ckb::helper::{Delegate, OmniEth, Tx, Xudt};

use crate::config::parse_type_ids;
use crate::{MAX_TRY, ROCKSDB_PATH, TYPE_IDS_PATH};

pub async fn delegate_smt_tx(ckb: &CkbRpcClient, kicker_key: H256, delegators_key: Vec<H256>) {
    let type_ids = parse_type_ids(TYPE_IDS_PATH);
    let metadata_type_id = type_ids.metadata_type_id.into_h256().unwrap();
    let checkpoint_type_id = type_ids.checkpoint_type_id.into_h256().unwrap();
    let delegate_smt_type_id = type_ids.delegate_smt_type_id.into_h256().unwrap();
    let xudt_owner = type_ids.xudt_owner.into_h256().unwrap();

    let mut delegate_cells = vec![];
    for (i, delegator_key) in delegators_key.into_iter().enumerate() {
        let omni_eth = OmniEth::new(delegator_key.clone());
        println!(
            "delegator{} ckb addres: {}\n",
            i,
            omni_eth.ckb_address().unwrap()
        );

        delegate_cells.push(
            Delegate::get_cell(
                ckb,
                Delegate::lock(&metadata_type_id, &omni_eth.address().unwrap()),
                Xudt::type_(&xudt_owner.pack()),
            )
            .await
            .unwrap()
            .expect("delegate AT cell not found"),
        );
    }

    let omni_eth = OmniEth::new(kicker_key.clone());
    println!("kicker ckb addres: {}\n", omni_eth.ckb_address().unwrap());

    let path = PathBuf::from(ROCKSDB_PATH);
    let smt = SmtManager::new(path);

    let (tx, _) = DelegateSmtTxBuilder::new(
        ckb,
        kicker_key,
        0,
        DelegateSmtTypeIds {
            metadata_type_id,
            checkpoint_type_id,
            delegate_smt_type_id,
            xudt_owner,
        },
        delegate_cells,
        smt,
    )
    .build_tx()
    .await
    .unwrap();

    let mut tx = Tx::new(ckb, tx);
    match tx.send().await {
        Ok(tx_hash) => println!("delegate smt tx hash: 0x{}", tx_hash),
        Err(e) => println!("{}", e),
    }

    println!("delegate smt tx ready");
    tx.wait_until_committed(1000, MAX_TRY).await.unwrap();
    println!("delegate smt tx committed");
}
