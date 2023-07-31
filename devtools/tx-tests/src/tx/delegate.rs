use ckb_types::H256;
use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;

use common::traits::tx_builder::IDelegateTxBuilder;
use common::types::tx_builder::{DelegateItem, EthAddress, StakeTypeIds};
use tx_builder::ckb::delegate::DelegateTxBuilder;
use tx_builder::ckb::helper::{OmniEth, Tx};

use crate::config::parse_type_ids;
use crate::TYPE_IDS_PATH;

pub async fn first_delegate_tx(
    ckb: &CkbRpcClient,
    delegator_key: H256,
    staker_eth_addr: EthAddress,
) {
    println!("first delegate");

    delegate_tx(
        ckb,
        delegator_key,
        vec![DelegateItem {
            staker: staker_eth_addr,
            is_increase: true,
            amount: 100,
            inauguration_epoch: 2,
            ..Default::default()
        }],
        0,
        true,
    )
    .await;
}

pub async fn add_delegate_tx(ckb: &CkbRpcClient, delegator_key: H256, staker_eth_addr: EthAddress) {
    println!("add delegate");

    delegate_tx(
        ckb,
        delegator_key,
        vec![DelegateItem {
            staker: staker_eth_addr,
            is_increase: true,
            amount: 10,
            inauguration_epoch: 2,
            ..Default::default()
        }],
        0,
        false,
    )
    .await;
}

pub async fn reedem_delegate_tx(
    ckb: &CkbRpcClient,
    delegator_key: H256,
    staker_eth_addr: EthAddress,
) {
    println!("redeem delegate");

    delegate_tx(
        ckb,
        delegator_key,
        vec![DelegateItem {
            staker: staker_eth_addr,
            is_increase: false,
            amount: 10,
            inauguration_epoch: 2,
            ..Default::default()
        }],
        0,
        false,
    )
    .await;
}

async fn delegate_tx(
    ckb: &CkbRpcClient,
    delegator_key: H256,
    delegates: Vec<DelegateItem>,
    current_epoch: u64,
    first_delegate: bool,
) {
    let type_ids = parse_type_ids(TYPE_IDS_PATH);

    let omni_eth = OmniEth::new(delegator_key.clone());
    println!(
        "delegatorr ckb addres: {}\n",
        omni_eth.ckb_address().unwrap()
    );

    let checkpoint_type_id = type_ids.checkpoint_type_id.into_h256().unwrap();
    let metadata_type_id = type_ids.metadata_type_id.into_h256().unwrap();
    let xudt_args = type_ids.xudt_owner.into_h256().unwrap();

    let tx = DelegateTxBuilder::new(
        ckb,
        StakeTypeIds {
            metadata_type_id,
            checkpoint_type_id,
            xudt_owner: xudt_args,
        },
        omni_eth.address().unwrap(),
        current_epoch,
        delegates,
    )
    .build_tx()
    .await
    .unwrap();

    let mut tx = Tx::new(ckb, tx);
    let script_groups = tx.gen_script_group().await.unwrap();
    let signer = omni_eth.signer().unwrap();

    if first_delegate {
        for group in script_groups.lock_groups.iter() {
            tx.sign(&signer, group.1).unwrap();
        }
    } else {
        for (i, group) in script_groups.lock_groups.iter().enumerate() {
            if i == 0 {
                println!("not sign; delegate AT cell: {:?}", group.1.input_indices);
            } else {
                println!("sign; other cell: {:?}", group.1.input_indices);
                tx.sign(&signer, group.1).unwrap();
            }
        }
    }

    match tx.send().await {
        Ok(tx_hash) => println!("delegate tx hash: 0x{}", tx_hash),
        Err(e) => println!("{}", e),
    }

    println!("delegate tx ready");
    tx.wait_until_committed(1000, 10).await.unwrap();
    println!("delegate tx committed");
}
