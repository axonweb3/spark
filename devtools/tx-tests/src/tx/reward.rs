use std::path::PathBuf;

use anyhow::Result;
use ckb_types::H256;

use common::traits::tx_builder::IRewardTxBuilder;
use common::types::tx_builder::RewardTypeIds;
use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;
use storage::SmtManager;
use tx_builder::ckb::helper::{OmniEth, Tx};
use tx_builder::ckb::reward::RewardTxBuilder;

use crate::config::parse_type_ids;
use crate::{MAX_TRY, ROCKSDB_PATH, TYPE_IDS_PATH};

pub async fn run_reward_tx(ckb: &CkbRpcClient, user_key: H256, current_epoch: u64) -> Result<()> {
    let type_ids = parse_type_ids(TYPE_IDS_PATH);

    let omni_eth = OmniEth::new(user_key.clone());
    let user = omni_eth.address().unwrap();

    let path = PathBuf::from(ROCKSDB_PATH);
    let smt = SmtManager::new(path);

    let tx = RewardTxBuilder::new(
        ckb,
        RewardTypeIds {
            selection_type_id:    type_ids.selection_type_id.into_h256().unwrap(),
            metadata_type_id:     type_ids.metadata_type_id.into_h256().unwrap(),
            checkpoint_type_id:   type_ids.checkpoint_type_id.into_h256().unwrap(),
            reward_smt_type_id:   type_ids.reward_smt_type_id.into_h256().unwrap(),
            stake_smt_type_id:    type_ids.stake_smt_type_id.into_h256().unwrap(),
            delegate_smt_type_id: type_ids.delegate_smt_type_id.into_h256().unwrap(),
            xudt_owner:           type_ids.xudt_owner.into_h256().unwrap(),
        },
        smt,
        user,
        current_epoch,
        1,
    )
    .await
    .build_tx()
    .await?;

    let mut tx = Tx::new(ckb, tx);

    let script_groups = tx.gen_script_group().await.unwrap();
    let signer = omni_eth.signer().unwrap();

    for (i, group) in script_groups.lock_groups.iter().enumerate() {
        if i <= 1 {
            println!(
                "not sign, reward cell or selection cell: {:?}",
                group.1.input_indices
            );
        } else {
            println!("sign, AT cell or ckb cell: {:?}", group.1.input_indices);
            tx.sign(&signer, group.1).unwrap();
        }
    }

    match tx.send().await {
        Ok(tx_hash) => println!("reward tx hash: 0x{}", tx_hash),
        Err(e) => println!("{}", e),
    }

    println!("reward tx ready");
    tx.wait_until_committed(1000, MAX_TRY).await.unwrap();
    println!("reward tx committed");

    Ok(())
}
