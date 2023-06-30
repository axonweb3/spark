#[cfg(test)]
mod tests {
    use ckb_types::h256;

    use common::traits::tx_builder::IWithdrawTxBuilder;
    use common::types::tx_builder::{Epoch, StakeTypeIds};
    use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;

    use crate::ckb::helper::ckb::{OmniEth, Tx};
    use crate::ckb::withdraw::WithdrawTxBuilder;

    // #[tokio::test]
    async fn _withdraw_test1_tx() {
        _withdraw_tx(1).await;
    }

    // #[tokio::test]
    async fn _withdraw_test2_tx() {
        _withdraw_tx(2).await;
    }

    async fn _withdraw_tx(current_epoch: Epoch) {
        let test_staker_key =
            h256!("0x13b08bb054d5dd04013156dced8ba2ce4d8cc5973e10d905a228ea1abc267e62");
        let xudt_args = h256!("0xfdaf95d57c615deaed3d7307d3f649b88d50a51f592a428f3815768e5ae3eab3");
        let checkpoint_type_id =
            h256!("0xfdaf95d57c615deaed3d7307d3f649b88d50a51f592a428f3815768e5ae3eab3");
        let metadata_type_id =
            h256!("0xfdaf95d57c615deaed3d7307d3f649b88d50a51f592a428f3815768e5ae3eab3");
        let ckb_client = CkbRpcClient::new("https://testnet.ckb.dev");

        let omni_eth = OmniEth::new(test_staker_key.clone());
        let staker_eth_addr = omni_eth.address().unwrap();
        let staker_ckb_addr = omni_eth.ckb_address().unwrap();
        println!("staker addr: {}", staker_ckb_addr);

        let tx = WithdrawTxBuilder::new(
            &ckb_client,
            StakeTypeIds {
                metadata_type_id,
                checkpoint_type_id,
                xudt_owner: xudt_args,
            },
            staker_eth_addr,
            current_epoch,
        )
        .build_tx()
        .await
        .unwrap();

        println!("tx: {}", tx);

        let mut tx = Tx::new(&ckb_client, tx);

        let script_groups = tx.gen_script_group().await.unwrap();
        let signer = omni_eth.signer().unwrap();

        for group in script_groups.lock_groups.iter() {
            println!("id: {:?}", group.1.input_indices);
            tx.sign(&signer, group.1).unwrap();
        }

        match tx.send().await {
            Ok(tx_hash) => println!("tx hash: 0x{}", tx_hash),
            Err(e) => println!("{}", e),
        }
    }
}
