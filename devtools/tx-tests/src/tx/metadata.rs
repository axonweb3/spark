use std::collections::HashMap;
use std::path::PathBuf;

use ckb_types::packed::WitnessArgs;
use ckb_types::prelude::{Entity, Unpack};
use ckb_types::{core::TransactionView, H256};
use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;
use storage::SmtManager;

use common::traits::tx_builder::IMetadataTxBuilder;
use common::types::axon_types::delegate::DelegateSmtCellData;
use common::types::axon_types::metadata::MetadataWitness as AMetadataWitness;
use common::types::tx_builder::MetadataTypeIds;
use common::utils::convert::{to_h160, to_u128};
use tx_builder::ckb::helper::{cell_collector::get_live_cell, Checkpoint, Tx};
use tx_builder::ckb::metadata::MetadataSmtTxBuilder;

use crate::config::parse_type_ids;
use crate::helper::smt::{generate_smt_root, to_root, verify_proof};
use crate::{MAX_TRY, ROCKSDB_PATH, TYPE_IDS_PATH};

pub async fn run_metadata_tx(ckb: &CkbRpcClient, kicker_key: H256, current_epoch: u64) {
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

    verify_old_delegat_smt(ckb, &tx, current_epoch).await;

    let mut tx = Tx::new(ckb, tx);
    match tx.send().await {
        Ok(tx_hash) => println!("metadata tx hash: 0x{}", tx_hash),
        Err(e) => println!("{}", e),
    }

    println!("metadata tx ready");
    tx.wait_until_committed(1000, MAX_TRY).await.unwrap();
    println!("metadata tx committed");
}

async fn verify_old_delegat_smt(ckb: &CkbRpcClient, tx: &TransactionView, current_epoch: u64) {
    println!("------------verify old delegate smt in metadata tx start----------");
    let old_delegate_smt_roots = {
        let delegate_smt_input = tx.inputs().get(2).unwrap();
        let delegate_smt_cell = get_live_cell(ckb, delegate_smt_input.previous_output(), true)
            .await
            .unwrap();
        let delegate_smt_data = DelegateSmtCellData::new_unchecked(
            delegate_smt_cell.data.unwrap().content.into_bytes(),
        );

        let mut new_top_roots = HashMap::new();
        for root in delegate_smt_data.smt_roots() {
            let staker = to_h160(&root.staker());
            let new_root = to_root(&root.root().as_bytes());
            new_top_roots.insert(staker.clone(), new_root);
            println!(
                "staker: {}, old top delegate smt root: {:?}",
                staker, new_root
            );
        }
        new_top_roots
    };

    let miners = {
        let metadata_witness = tx.witnesses().get(0).unwrap();
        let metadata_witness = WitnessArgs::new_unchecked(metadata_witness.unpack());
        let metadata_witness = metadata_witness.input_type().to_opt().unwrap().unpack();
        let metadata_witness = AMetadataWitness::new_unchecked(metadata_witness);
        metadata_witness.smt_election_info().n2().miners()
    };

    for miner in miners {
        let staker = to_h160(&miner.staker());
        let root = old_delegate_smt_roots.get(&staker).unwrap().to_owned();
        let proof = miner.delegate_epoch_proof().raw_data().to_vec();
        println!("proof: {:?}", proof);

        let mut leaves = HashMap::new();
        for delegate in miner.delegate_infos() {
            let addr = to_h160(&delegate.addr());
            let amount = to_u128(&delegate.amount());
            println!(
                "staker: {}, delegator: {}, amount: {}",
                staker, addr, amount
            );
            leaves.insert(addr.0.into(), amount);
        }

        let bottom_root = generate_smt_root(leaves);
        println!("bottom root: {:?}", bottom_root);

        let ok = verify_proof(root, proof, current_epoch + 2, bottom_root);
        println!("verify result: {}", ok);
    }
    println!("------------verify old delegate smt in metadata tx end-----------");
}
