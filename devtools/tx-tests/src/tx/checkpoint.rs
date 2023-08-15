use ckb_types::H256;

use common::traits::tx_builder::ICheckpointTxBuilder;
use common::types::tx_builder::{Checkpoint, CheckpointProof, CheckpointTypeIds, ProposeCount};
use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;
use tx_builder::ckb::checkpoint::CheckpointTxBuilder;
use tx_builder::ckb::helper::{OmniEth, Tx};

use crate::config::parse_type_ids;
use crate::config::types::PrivKeys;
use crate::mock::{mock_axon_proof_v2, mock_axon_proposal_v2};
use crate::{MAX_TRY, TYPE_IDS_PATH};

pub async fn run_checkpoint_tx(ckb: &CkbRpcClient, priv_keys: PrivKeys, epoch: u64) {
    let kicker_key = priv_keys.staker_privkeys[0].clone().into_h256().unwrap();
    let omni_eth = OmniEth::new(kicker_key.clone());
    println!("kicker ckb addres: {}\n", omni_eth.ckb_address().unwrap());

    let mut staker_privkeys = vec![];
    let mut propose_count = vec![];
    for staker_privkey in priv_keys.staker_privkeys.into_iter() {
        let privkey = staker_privkey.clone().into_h256().unwrap();
        staker_privkeys.push(privkey);

        propose_count.push(ProposeCount {
            proposer: omni_eth.address().unwrap(),
            count:    100,
        });
    }

    checkpoint_tx(
        ckb,
        kicker_key,
        1,
        Checkpoint {
            epoch,
            period: 0,
            latest_block_height: 10,
            timestamp: 11111,
            propose_count,
            ..Default::default()
        },
        CheckpointProof {
            proof:    mock_axon_proof_v2(&staker_privkeys),
            proposal: mock_axon_proposal_v2(),
        },
    )
    .await;
}

pub async fn checkpoint_tx(
    ckb: &CkbRpcClient,
    kicker_key: H256,
    epoch_len: u64,
    new_checkpoint: Checkpoint,
    proof: CheckpointProof,
) {
    let type_ids = parse_type_ids(TYPE_IDS_PATH);
    let metadata_type_id = type_ids.metadata_type_id.into_h256().unwrap();
    let checkpoint_type_id = type_ids.checkpoint_type_id.into_h256().unwrap();

    let tx = CheckpointTxBuilder::new(
        ckb,
        kicker_key.clone(),
        CheckpointTypeIds {
            metadata_type_id,
            checkpoint_type_id,
        },
        epoch_len,
        new_checkpoint,
        proof,
    )
    .await
    .build_tx()
    .await
    .unwrap();

    let mut tx = Tx::new(ckb, tx);
    let script_groups = tx.gen_script_group().await.unwrap();
    let signer = OmniEth::new(kicker_key).signer().unwrap();
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
        Ok(tx_hash) => println!("checkpoint tx hash: 0x{}", tx_hash),
        Err(e) => println!("{}", e),
    }

    println!("checkpoint tx ready");
    tx.wait_until_committed(1000, MAX_TRY).await.unwrap();
    println!("checkpoint tx committed");
}
