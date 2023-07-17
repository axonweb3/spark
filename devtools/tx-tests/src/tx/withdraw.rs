use common::traits::tx_builder::IWithdrawTxBuilder;
use common::types::tx_builder::StakeTypeIds;
use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;
use tx_builder::ckb::helper::{OmniEth, Tx};
use tx_builder::ckb::withdraw::WithdrawTxBuilder;

use crate::config::parse_file;
use crate::config::types::{PrivKeys, TypeIds};
use crate::{PRIV_KEYS_PATH, TYPE_IDS_PATH};

pub async fn withdraw_tx(ckb: &CkbRpcClient) {
    let priv_keys: PrivKeys = parse_file(PRIV_KEYS_PATH);
    let test_staker_key = priv_keys.staker_privkeys[0].clone().into_h256().unwrap();
    let omni_eth = OmniEth::new(test_staker_key.clone());
    println!("staker0 ckb addres: {}\n", omni_eth.ckb_address().unwrap());

    let type_ids: TypeIds = parse_file(TYPE_IDS_PATH);
    let checkpoint_type_id = type_ids.checkpoint_type_id.into_h256().unwrap();
    let metadata_type_id = type_ids.metadata_type_id.into_h256().unwrap();
    let xudt_args = type_ids.xudt_owner.into_h256().unwrap();

    let tx = WithdrawTxBuilder::new(
        ckb,
        StakeTypeIds {
            metadata_type_id,
            checkpoint_type_id,
            xudt_owner: xudt_args,
        },
        omni_eth.address().unwrap(),
        2,
    )
    .build_tx()
    .await
    .unwrap();

    let mut tx = Tx::new(ckb, tx);
    let script_groups = tx.gen_script_group().await.unwrap();
    let signer = omni_eth.signer().unwrap();

    for (i, group) in script_groups.lock_groups.iter().enumerate() {
        if i == 0 {
            println!("not sign; withdraw AT cell: {:?}", group.1.input_indices);
        } else {
            println!("sign; other cell: {:?}", group.1.input_indices);
            tx.sign(&signer, group.1).unwrap();
        }
    }

    match tx.send().await {
        Ok(tx_hash) => println!("tx hash: 0x{}", tx_hash),
        Err(e) => println!("{}", e),
    }
}
