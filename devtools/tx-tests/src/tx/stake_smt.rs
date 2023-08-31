use ckb_types::packed::WitnessArgs;
use ckb_types::prelude::Entity;
use ckb_types::prelude::Unpack;
use ckb_types::{core::TransactionView, prelude::Pack, H256};
use common::types::axon_types::stake::StakeSmtWitness;
use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;

use common::traits::smt::StakeSmtStorage;
use common::traits::tx_builder::IStakeSmtTxBuilder;
use common::types::axon_types::stake::StakeSmtCellData;
use common::types::tx_builder::StakeSmtTypeIds;
use storage::SmtManager;
use tx_builder::ckb::helper::{OmniEth, Stake, Tx, Xudt};
use tx_builder::ckb::stake_smt::StakeSmtTxBuilder;

use crate::config::parse_type_ids;
use crate::helper::smt::{generate_smt_root, to_root, verify_proof};
use crate::{MAX_TRY, TYPE_IDS_PATH};

pub async fn stake_smt_tx(
    ckb: &CkbRpcClient,
    smt: &SmtManager,
    kicker_key: H256,
    stakers_key: Vec<H256>,
    current_epoch: u64,
) {
    let type_ids = parse_type_ids(TYPE_IDS_PATH);
    let metadata_type_id = type_ids.metadata_type_id.into_h256().unwrap();
    let checkpoint_type_id = type_ids.checkpoint_type_id.into_h256().unwrap();
    let stake_smt_type_id = type_ids.stake_smt_type_id.into_h256().unwrap();
    let xudt_owner = type_ids.xudt_owner.into_h256().unwrap();

    let mut stake_cells = vec![];
    for staker_key in stakers_key.into_iter() {
        let omni_eth = OmniEth::new(staker_key.clone());
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

    let (tx, _) = StakeSmtTxBuilder::new(
        ckb,
        smt,
        kicker_key,
        current_epoch,
        StakeSmtTypeIds {
            metadata_type_id,
            checkpoint_type_id,
            stake_smt_type_id,
            xudt_owner,
        },
        stake_cells,
    )
    .build_tx()
    .await
    .unwrap();

    verify_new_stake_smt(smt, &tx, current_epoch).await;

    let mut tx = Tx::new(ckb, tx);
    match tx.send().await {
        Ok(tx_hash) => println!("stake smt tx hash: 0x{}", tx_hash),
        Err(e) => println!("{}", e),
    }

    println!("stake smt tx ready");
    tx.wait_until_committed(1000, MAX_TRY).await.unwrap();
    println!("stake smt tx committed");
}

async fn verify_new_stake_smt(smt: &SmtManager, tx: &TransactionView, current_epoch: u64) {
    println!("------------verify new stake smt start-----------");
    let new_top_root = {
        let smt_data = tx.outputs_data().get(0).unwrap();
        let smt_data = StakeSmtCellData::new_unchecked(smt_data.unpack());
        let new_root = to_root(&smt_data.smt_root().as_bytes());
        println!("new top root: {:?}", new_root);
        new_root
    };

    let new_epoch_proof = {
        let smt_witness = tx.witnesses().get(0).unwrap();
        let smt_witness = WitnessArgs::new_unchecked(smt_witness.unpack());
        let smt_witness = smt_witness.input_type().to_opt().unwrap().unpack();
        let new_epoch_proof = StakeSmtWitness::new_unchecked(smt_witness)
            .update_info()
            .new_epoch_proof()
            .raw_data()
            .to_vec();
        println!("new epoch proof: {:?}", new_epoch_proof);
        new_epoch_proof
    };

    let bottom_root_created = {
        let leaves = StakeSmtStorage::get_sub_leaves(smt, current_epoch + 2)
            .await
            .unwrap();
        for (k, v) in leaves.iter() {
            println!("stake smt leaves: {:?} {}", k.0, v);
        }

        let root = generate_smt_root(leaves);
        println!("bottom root created from kv: {:?}", root);
        root
    };

    let bottom_root_gotten = StakeSmtStorage::get_sub_root(smt, current_epoch + 2)
        .await
        .unwrap()
        .unwrap();
    println!("bottom root gotten from smt: {:?}", bottom_root_gotten);

    assert_eq!(bottom_root_created, bottom_root_gotten);

    let ok = verify_proof(
        new_top_root,
        new_epoch_proof,
        current_epoch + 2,
        bottom_root_gotten,
    );
    println!("verify result: {}", ok);
    println!("------------verify new stake smt end-----------");
}
