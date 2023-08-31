use std::collections::HashMap;

use ckb_types::packed::WitnessArgs;
use ckb_types::prelude::{Entity, Unpack};
use ckb_types::{core::TransactionView, prelude::Pack, H256};
use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;
use storage::SmtManager;

use common::traits::smt::DelegateSmtStorage;
use common::traits::tx_builder::IDelegateSmtTxBuilder;
use common::types::axon_types::delegate::{DelegateSmtCellData, DelegateSmtWitness};
use common::types::tx_builder::DelegateSmtTypeIds;
use common::utils::convert::to_h160;
use tx_builder::ckb::delegate_smt::DelegateSmtTxBuilder;
use tx_builder::ckb::helper::{Delegate, OmniEth, Tx, Xudt};

use crate::config::parse_type_ids;
use crate::helper::smt::{generate_smt_root, to_root, verify_proof};
use crate::{MAX_TRY, TYPE_IDS_PATH};

pub async fn delegate_smt_tx(
    ckb: &CkbRpcClient,
    smt: &SmtManager,
    kicker_key: H256,
    delegators_key: Vec<H256>,
    current_epoch: u64,
) {
    let type_ids = parse_type_ids(TYPE_IDS_PATH);
    let metadata_type_id = type_ids.metadata_type_id.into_h256().unwrap();
    let checkpoint_type_id = type_ids.checkpoint_type_id.into_h256().unwrap();
    let delegate_smt_type_id = type_ids.delegate_smt_type_id.into_h256().unwrap();
    let xudt_owner = type_ids.xudt_owner.into_h256().unwrap();

    let mut delegate_cells = vec![];
    for delegator_key in delegators_key.into_iter() {
        let omni_eth = OmniEth::new(delegator_key.clone());
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

    let (tx, _) = DelegateSmtTxBuilder::new(
        ckb,
        smt,
        kicker_key,
        current_epoch,
        DelegateSmtTypeIds {
            metadata_type_id,
            checkpoint_type_id,
            delegate_smt_type_id,
            xudt_owner,
        },
        delegate_cells,
    )
    .build_tx()
    .await
    .unwrap();

    verify_new_delegate_smt(smt, &tx, current_epoch).await;

    let mut tx = Tx::new(ckb, tx);
    match tx.send().await {
        Ok(tx_hash) => println!("delegate smt tx hash: 0x{}", tx_hash),
        Err(e) => println!("{}", e),
    }

    println!("delegate smt tx ready");
    tx.wait_until_committed(1000, MAX_TRY).await.unwrap();
    println!("delegate smt tx committed");
}

async fn verify_new_delegate_smt(smt: &SmtManager, tx: &TransactionView, current_epoch: u64) {
    println!("------------verify new delegate smt start-----------");
    let new_delegate_smt_roots = {
        let delegate_smt_data = tx.outputs_data().get(0).unwrap();
        let delegate_smt_data = DelegateSmtCellData::new_unchecked(delegate_smt_data.unpack());

        let mut new_top_roots = HashMap::new();
        for root in delegate_smt_data.smt_roots() {
            let staker = to_h160(&root.staker());
            let new_root = to_root(&root.root().as_bytes());
            new_top_roots.insert(staker.clone(), new_root);
            println!(
                "staker: {}, new top delegate smt root: {:?}",
                staker, new_root
            );
        }
        new_top_roots
    };

    let stake_group = {
        let smt_witness = tx.witnesses().get(0).unwrap();
        let smt_witness = WitnessArgs::new_unchecked(smt_witness.unpack());
        let smt_witness = smt_witness.input_type().to_opt().unwrap().unpack();
        DelegateSmtWitness::new_unchecked(smt_witness)
            .update_info()
            .all_stake_group_infos()
    };

    for group in stake_group {
        let staker = to_h160(&group.staker());
        let root = new_delegate_smt_roots.get(&staker).unwrap().to_owned();
        let proof = group.delegate_new_epoch_proof().raw_data().to_vec();
        println!("staker: {}, delegate proof: {:?}", staker, proof);

        let bottom_root_created = {
            let leaves =
                DelegateSmtStorage::get_sub_leaves(smt, current_epoch + 2, staker.0.into())
                    .await
                    .unwrap();
            for (k, v) in leaves.iter() {
                println!("delegate smt leaves: {} {}", k, v);
            }

            let root = generate_smt_root(leaves);
            println!("bottom root created from kv: {:?}", root);
            root
        };

        let bottom_root_gotten =
            DelegateSmtStorage::get_sub_root(smt, current_epoch + 2, staker.0.into())
                .await
                .unwrap()
                .unwrap();
        println!("bottom root gotten from smt: {:?}", bottom_root_gotten);

        // It sometimes fails
        if bottom_root_created != bottom_root_gotten {
            println!("error: invalid bottom root!");
        }

        let ok = verify_proof(root, proof, current_epoch + 2, bottom_root_gotten);
        println!("verify result: {}", ok);
    }

    println!("------------verify new delegate smt end-----------");
}
