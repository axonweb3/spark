use anyhow::Result;
use ckb_types::H256;
use molecule::prelude::Entity;
use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;

use common::traits::tx_builder::IStakeTxBuilder;
use common::types::axon_types::basic::{Byte48, Byte65};
use common::types::tx_builder::{DelegateRequirement, FirstStakeInfo, StakeItem, StakeTypeIds};
use tx_builder::ckb::helper::{OmniEth, Tx};
use tx_builder::ckb::stake::StakeTxBuilder;

use crate::config::parse_type_ids;
use crate::helper::signer::{EthSigner, UnlockMode};
use crate::mock::gen_bls_keypair;
use crate::{MAX_TRY, TYPE_IDS_PATH};

pub async fn first_stake_tx(ckb: &CkbRpcClient, staker_key: H256, amount: u128) {
    println!("first stake");

    let bls_pub_key = gen_bls_keypair(staker_key.as_bytes()).1;

    stake_tx(
        ckb,
        staker_key,
        StakeItem {
            is_increase: true,
            amount,
            inauguration_epoch: 2,
        },
        0,
        Some(FirstStakeInfo {
            l1_pub_key:  Byte65::default(),
            bls_pub_key: Byte48::new_unchecked(bls_pub_key.into()),
            delegate:    DelegateRequirement {
                commission_rate:    20,
                maximum_delegators: 1,
                threshold:          0,
            },
        }),
    )
    .await
    .unwrap();
}

pub async fn add_stake_tx(
    ckb: &CkbRpcClient,
    staker_key: H256,
    amount: u128,
    current_epoch: u64,
) -> Result<()> {
    println!("add stake");

    stake_tx(
        ckb,
        staker_key,
        StakeItem {
            is_increase: true,
            amount,
            inauguration_epoch: current_epoch + 2,
        },
        current_epoch,
        None,
    )
    .await
}

pub async fn redeem_stake_tx(
    ckb: &CkbRpcClient,
    staker_key: H256,
    amount: u128,
    current_epoch: u64,
) -> Result<()> {
    println!("redeem stake");

    stake_tx(
        ckb,
        staker_key,
        StakeItem {
            is_increase: false,
            amount,
            inauguration_epoch: current_epoch + 2,
        },
        current_epoch,
        None,
    )
    .await
}

async fn stake_tx(
    ckb: &CkbRpcClient,
    staker_key: H256,
    stake_item: StakeItem,
    current_epoch: u64,
    first_stake_info: Option<FirstStakeInfo>,
) -> Result<()> {
    let type_ids = parse_type_ids(TYPE_IDS_PATH);
    let first_stake = first_stake_info.is_some();
    let checkpoint_type_id = type_ids.checkpoint_type_id.into_h256().unwrap();
    let metadata_type_id = type_ids.metadata_type_id.into_h256().unwrap();
    let xudt_args = type_ids.xudt_owner.into_h256().unwrap();

    let omni_eth = OmniEth::new(staker_key.clone());

    let tx = StakeTxBuilder::new(
        ckb,
        StakeTypeIds {
            metadata_type_id,
            checkpoint_type_id,
            xudt_owner: xudt_args,
        },
        omni_eth.address().unwrap(),
        current_epoch,
        stake_item,
        first_stake_info,
    )
    .build_tx()
    .await?;

    // let json_tx = ckb_jsonrpc_types::TransactionView::from(tx);
    // println!("{}", serde_json::to_string_pretty(&json_tx).unwrap());

    let mut tx = Tx::new(ckb, tx);
    let script_groups = tx.gen_script_group().await.unwrap();
    let omni_signer = omni_eth.signer().unwrap();
    let eth_signer = EthSigner::new(staker_key, UnlockMode::Stake);

    if first_stake {
        for group in script_groups.lock_groups.iter() {
            tx.sign(&omni_signer, group.1).unwrap();
        }
    } else {
        for (i, group) in script_groups.lock_groups.iter().enumerate() {
            if i == 0 {
                println!("stake AT cell: {:?}", group.1.input_indices);
                tx.sign(&eth_signer, group.1).unwrap();
            } else {
                println!("other cells: {:?}", group.1.input_indices);
                tx.sign(&omni_signer, group.1).unwrap();
            }
        }
    }

    match tx.send().await {
        Ok(tx_hash) => println!("stake tx hash: 0x{}", tx_hash),
        Err(e) => println!("{}", e),
    }

    println!("stake tx ready");
    tx.wait_until_committed(1000, MAX_TRY).await.unwrap();
    println!("stake tx committed");

    Ok(())
}
