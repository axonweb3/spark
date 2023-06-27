use ckb_sdk::unlock::ScriptSigner;
use ckb_types::H160;
use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;

use common::traits::tx_builder::IDelegateTxBuilder;
use common::types::tx_builder::{CkbNetwork, DelegateItem, StakeTypeIds};
use tx_builder::ckb::delegate::DelegateTxBuilder;
use tx_builder::ckb::utils::omni::{omni_eth_address, omni_eth_ckb_address, omni_eth_signer};
use tx_builder::ckb::utils::tx::{gen_script_group, send_tx};

use crate::config::parse_file;
use crate::config::types::{PrivKeys, TypeIds as CTypeIds};
use crate::{PRIV_KEYS_PATH, TYPE_IDS_PATH};

fn stakers() -> Vec<H160> {
    let priv_keys: PrivKeys = parse_file(PRIV_KEYS_PATH);
    let mut stakers = vec![];

    for priv_key in priv_keys.staker_privkeys {
        let test_delegator_key = priv_key.into_h256().unwrap();
        stakers.push(omni_eth_address(test_delegator_key.clone()).unwrap())
    }
    stakers
}

pub async fn first_delegate_tx(ckb: CkbNetwork<CkbRpcClient>) {
    println!("first delegate");

    delegate_tx(
        ckb,
        vec![DelegateItem {
            staker: stakers()[0].clone(),
            is_increase: true,
            amount: 10,
            inauguration_epoch: 2,
            ..Default::default()
        }],
        0,
        true,
    )
    .await;
}

pub async fn add_delegate_tx(ckb: CkbNetwork<CkbRpcClient>) {
    println!("add delegate");

    delegate_tx(
        ckb,
        vec![DelegateItem {
            staker: stakers()[0].clone(),
            is_increase: true,
            amount: 5,
            inauguration_epoch: 2,
            ..Default::default()
        }],
        0,
        false,
    )
    .await;
}

pub async fn reedem_delegate_tx(ckb: CkbNetwork<CkbRpcClient>) {
    println!("redeem delegate");

    delegate_tx(
        ckb,
        vec![DelegateItem {
            staker: stakers()[0].clone(),
            is_increase: false,
            amount: 1,
            inauguration_epoch: 2,
            ..Default::default()
        }],
        0,
        false,
    )
    .await;
}

async fn delegate_tx(
    ckb: CkbNetwork<CkbRpcClient>,
    delegates: Vec<DelegateItem>,
    current_epoch: u64,
    first_delegate: bool,
) {
    let priv_keys: PrivKeys = parse_file(PRIV_KEYS_PATH);
    let test_delegator_key = priv_keys.delegator_privkeys[1].clone().into_h256().unwrap();
    println!(
        "delegatorr ckb addres: {}\n",
        omni_eth_ckb_address(&ckb.network_type, test_delegator_key.clone()).unwrap()
    );

    let type_ids: CTypeIds = parse_file(TYPE_IDS_PATH);
    let checkpoint_type_id = type_ids.checkpoint_type_id.into_h256().unwrap();
    let metadata_type_id = type_ids.metadata_type_id.into_h256().unwrap();
    let xudt_args = type_ids.xudt_owner.into_h256().unwrap();

    let mut tx = DelegateTxBuilder::new(
        ckb.clone(),
        StakeTypeIds {
            metadata_type_id,
            checkpoint_type_id,
            xudt_owner: xudt_args,
        },
        omni_eth_address(test_delegator_key.clone()).unwrap(),
        current_epoch,
        delegates,
    )
    .build_tx()
    .await
    .unwrap();

    let signer = omni_eth_signer(test_delegator_key).unwrap();
    let script_groups = gen_script_group(&ckb.client, &tx).await.unwrap();

    if first_delegate {
        for group in script_groups.lock_groups.iter() {
            tx = signer.sign_tx(&tx, group.1).unwrap();
        }
    } else {
        let mut first_group = true;
        for group in script_groups.lock_groups.iter() {
            if !first_group {
                println!("sign; not delegate id: {:?}", group.1.input_indices);
                tx = signer.sign_tx(&tx, group.1).unwrap();
            } else {
                println!("not sign; delegate id: {:?}", group.1.input_indices);
            }
            first_group = false;
        }
    }

    match send_tx(&ckb.client, &tx.data().into()).await {
        Ok(tx_hash) => println!("tx hash: 0x{}", tx_hash),
        Err(e) => println!("{}", e),
    }

    println!("\ntx: {}", tx);
}
