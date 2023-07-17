use std::{path::PathBuf, vec};

use common::traits::smt::{DelegateSmtStorage, ProposalSmtStorage, StakeSmtStorage};
use common::traits::tx_builder::IRewardTxBuilder;
use common::types::smt::{Staker, UserAmount};
use common::types::tx_builder::{Checkpoint, Metadata, RewardInfo, RewardTypeIds};
use common::utils::convert::to_eth_h160;
use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;
use storage::SmtManager;
use tx_builder::ckb::helper::{OmniEth, Tx};
use tx_builder::ckb::reward::RewardTxBuilder;

use crate::config::parse_file;
use crate::config::types::{PrivKeys, TypeIds as CTypeIds};
use crate::tx::init::_init_tx;
use crate::{PRIV_KEYS_PATH, TYPE_IDS_PATH};

static ROCKSDB_PATH: &str = "./free-space/smt/reward";

pub async fn reward_tx(ckb: &CkbRpcClient) {
    let priv_keys: PrivKeys = parse_file(PRIV_KEYS_PATH);

    let test_seeder_key = priv_keys.seeder_privkey.into_h256().unwrap();
    println!(
        "seeder ckb addres: {}\n",
        OmniEth::new(test_seeder_key.clone()).ckb_address().unwrap()
    );

    let tx = _init_tx(
        ckb,
        test_seeder_key,
        Checkpoint {
            epoch: 0,
            period: 0,
            latest_block_height: 10,
            timestamp: 11111,
            ..Default::default()
        },
        Metadata {
            epoch_len: 100,
            period_len: 100,
            quorum: 10,
            validators: vec![],
            ..Default::default()
        },
        vec![],
    )
    .await;

    tx.wait_until_committed(1000, 10).await.unwrap();
    println!("init tx committed");

    let test_staker_key = priv_keys.staker_privkeys[0].clone().into_h256().unwrap();
    let omni_eth = OmniEth::new(test_staker_key.clone());
    println!("staker ckb addres: {}\n", omni_eth.ckb_address().unwrap());
    let staker = omni_eth.address().unwrap();

    let type_ids: CTypeIds = parse_file(TYPE_IDS_PATH);

    let path = PathBuf::from(ROCKSDB_PATH);
    let smt = SmtManager::new(path);
    mock_smt(&smt, to_eth_h160(&staker)).await;

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
            base_reward:               100,
            half_reward_cycle:         200,
            theoretical_propose_count: 30,
            epoch_count:               10,
        },
        staker,
        5,
    )
    .build_tx()
    .await
    .unwrap();

    let mut tx = Tx::new(ckb, tx);

    let script_groups = tx.gen_script_group().await.unwrap();
    let signer = omni_eth.signer().unwrap();

    for (i, group) in script_groups.lock_groups.iter().enumerate() {
        if i <= 1 {
            println!("not sign: {:?}", group.1.input_indices);
        } else {
            println!("sign: {:?}", group.1.input_indices);
            tx.sign(&signer, group.1).unwrap();
        }
    }

    match tx.send().await {
        Ok(tx_hash) => println!("tx hash: 0x{}", tx_hash),
        Err(e) => println!("{}", e),
    }
}

async fn mock_smt(smt: &SmtManager, staker: Staker) {
    mock_stake_smt(smt, staker).await;
    mock_delegate_smt(smt, staker).await;
    mock_proposal_smt(smt, staker).await;
}

async fn mock_stake_smt(smt: &SmtManager, staker: Staker) {
    let epoch = 2;
    let amount = 100u128;

    let amounts = vec![UserAmount {
        user: staker,
        amount,
        is_increase: true,
    }];

    StakeSmtStorage::insert(smt, epoch, amounts).await.unwrap();
}

async fn mock_delegate_smt(smt: &SmtManager, staker: Staker) {
    let delegator = [6u8; 20].into();
    let epoch = 2;
    let amount = 100u128;

    let delegators = vec![UserAmount {
        user: delegator,
        amount,
        is_increase: true,
    }];

    DelegateSmtStorage::insert(smt, epoch, staker, delegators.clone())
        .await
        .unwrap();
}

async fn mock_proposal_smt(smt: &SmtManager, validator: Staker) {
    let epoch = 0;
    let proposal_count = 10;

    let proposals = vec![(validator, proposal_count)];

    ProposalSmtStorage::insert(smt, epoch, proposals)
        .await
        .unwrap();
}
