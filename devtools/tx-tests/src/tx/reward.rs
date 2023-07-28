use std::path::PathBuf;

use common::traits::tx_builder::IRewardTxBuilder;
use common::types::tx_builder::{RewardInfo, RewardTypeIds};
use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;
use storage::SmtManager;
use tx_builder::ckb::helper::{OmniEth, Tx};
use tx_builder::ckb::reward::RewardTxBuilder;

use crate::config::parse_file;
use crate::config::types::{PrivKeys, TypeIds as CTypeIds};
use crate::{PRIV_KEYS_PATH, TYPE_IDS_PATH};

static ROCKSDB_PATH: &str = "./free-space/smt";

pub async fn reward_tx(ckb: &CkbRpcClient) {
    let priv_keys: PrivKeys = parse_file(PRIV_KEYS_PATH);

    let test_seeder_key = priv_keys.seeder_privkey.into_h256().unwrap();
    println!(
        "seeder ckb addres: {}\n",
        OmniEth::new(test_seeder_key.clone()).ckb_address().unwrap()
    );

    let test_staker_key = priv_keys.staker_privkeys[0].clone().into_h256().unwrap();
    let omni_eth = OmniEth::new(test_staker_key.clone());
    println!("staker ckb addres: {}\n", omni_eth.ckb_address().unwrap());
    let staker = omni_eth.address().unwrap();

    let type_ids: CTypeIds = parse_file(TYPE_IDS_PATH);

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
        RewardInfo {
            base_reward:           1000,
            half_reward_cycle:     200,
            minimum_propose_count: 100,
            propose_discount_rate: 95,
            epoch_count:           1,
        },
        staker,
        2,
    )
    .build_tx()
    .await
    .unwrap();

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
            println!("sign, other cell: {:?}", group.1.input_indices);
            tx.sign(&signer, group.1).unwrap();
        }
    }

    match tx.send().await {
        Ok(tx_hash) => println!("tx hash: 0x{}", tx_hash),
        Err(e) => println!("{}", e),
    }
}
