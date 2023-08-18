use ckb_types::H256;

use common::traits::tx_builder::IWithdrawTxBuilder;
use common::types::tx_builder::StakeTypeIds;
use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;
use tx_builder::ckb::helper::{OmniEth, Tx};
use tx_builder::ckb::withdraw::WithdrawTxBuilder;

use crate::config::parse_type_ids;
use crate::{MAX_TRY, TYPE_IDS_PATH};

pub async fn run_withdraw_tx(ckb: &CkbRpcClient, user_key: H256, current_epoch: u64) {
    let type_ids = parse_type_ids(TYPE_IDS_PATH);

    let omni_eth = OmniEth::new(user_key.clone());
    println!("staker0 ckb addres: {}\n", omni_eth.ckb_address().unwrap());

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
        current_epoch,
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
        Ok(tx_hash) => println!("withdraw tx hash: 0x{}", tx_hash),
        Err(e) => println!("{}", e),
    }

    println!("withdraw tx ready");
    tx.wait_until_committed(1000, MAX_TRY).await.unwrap();
    println!("withdraw tx committed");
}
