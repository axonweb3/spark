use common::traits::tx_builder::ICheckpointTxBuilder;
use common::types::tx_builder::{Checkpoint, CheckpointProof, CheckpointTypeIds};
use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;
use tx_builder::ckb::checkpoint::CheckpointTxBuilder;
use tx_builder::ckb::helper::{OmniEth, Tx};

use crate::config::parse_file;
use crate::config::types::{PrivKeys, TypeIds as CTypeIds};
use crate::mock::{mock_axon_proof_v2, mock_axon_proposal_v2};
use crate::{PRIV_KEYS_PATH, TYPE_IDS_PATH};

pub async fn checkpoint_tx(ckb: &CkbRpcClient) {
    let priv_keys: PrivKeys = parse_file(PRIV_KEYS_PATH);
    let test_kicker_key = priv_keys.staker_privkeys[0].clone().into_h256().unwrap();
    let omni_eth = OmniEth::new(test_kicker_key.clone());
    println!("kicker ckb addres: {}\n", omni_eth.ckb_address().unwrap());

    let mut staker_privkeys = vec![];
    for staker_privkey in priv_keys.staker_privkeys.into_iter() {
        let privkey = staker_privkey.clone().into_h256().unwrap();
        staker_privkeys.push(privkey);
    }

    let type_ids: CTypeIds = parse_file(TYPE_IDS_PATH);
    let metadata_type_id = type_ids.metadata_type_id.into_h256().unwrap();
    let checkpoint_type_id = type_ids.checkpoint_type_id.into_h256().unwrap();

    let tx = CheckpointTxBuilder::new(
        ckb,
        test_kicker_key.clone(),
        CheckpointTypeIds {
            metadata_type_id,
            checkpoint_type_id,
        },
        100,
        Checkpoint {
            epoch: 0,
            period: 1,
            latest_block_height: 10,
            timestamp: 11111,
            ..Default::default()
        },
        CheckpointProof {
            proof:    mock_axon_proof_v2(&staker_privkeys),
            proposal: mock_axon_proposal_v2(),
        },
    )
    .await
    .build_tx()
    .await
    .unwrap();

    let mut tx = Tx::new(ckb, tx);
    let script_groups = tx.gen_script_group().await.unwrap();
    let signer = omni_eth.signer().unwrap();
    let mut first_group = true;

    for group in script_groups.lock_groups.iter() {
        if !first_group {
            println!("sign; not checkpoint id: {:?}", group.1.input_indices);
            tx.sign(&signer, group.1).unwrap();
        } else {
            println!("not sign; checkpoint id: {:?}", group.1.input_indices);
        }
        first_group = false;
    }

    match tx.send().await {
        Ok(tx_hash) => println!("tx hash: 0x{}", tx_hash),
        Err(e) => println!("{}", e),
    }

    println!("\ntx: {}", tx.inner());
}
