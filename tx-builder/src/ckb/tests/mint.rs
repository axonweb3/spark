#[cfg(test)]
mod tests {
    use ckb_types::h256;

    use common::traits::tx_builder::IMintTxBuilder;
    use common::types::tx_builder::{CkbNetwork, NetworkType};
    use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;

    use crate::ckb::mint::MintTxBuilder;
    use crate::ckb::utils::omni::omni_eth_address;
    use crate::ckb::utils::tx::send_tx;

    // #[tokio::test]
    async fn _send_mint_tx() {
        let test_seeder_key =
            h256!("0x13b08bb054d5dd04013156dced8ba2ce4d8cc5973e10d905a228ea1abc267e61");
        let test_staker1_key =
            h256!("0x13b08bb054d5dd04013156dced8ba2ce4d8cc5973e10d905a228ea1abc267e60");
        let test_staker2_key =
            h256!("0x13b08bb054d5dd04013156dced8ba2ce4d8cc5973e10d905a228ea1abc267e62");
        let selection_type_id =
            h256!("0xcefb1b795b553e93e791cd6471ddbd053d83fdb53db5b0e615001e47d6b10c76");
        let issue_type_id =
            h256!("0x9c043ae89b1c76d10751f4af8fb9054f51cdfd7243bf3435a0727b9dd9f180ef");
        let ckb_client = CkbRpcClient::new("https://testnet.ckb.dev");

        let tx = MintTxBuilder::new(
            CkbNetwork {
                network_type: NetworkType::Testnet,
                client:       ckb_client.clone(),
            },
            test_seeder_key,
            vec![
                (omni_eth_address(test_staker1_key).unwrap(), 100),
                (omni_eth_address(test_staker2_key).unwrap(), 200),
            ],
            selection_type_id,
            issue_type_id,
        )
        .build_tx()
        .await
        .unwrap();

        // println!("tx: {}", tx);

        match send_tx(&ckb_client, &tx.data().into()).await {
            Ok(tx_hash) => println!("tx hash: 0x{}", tx_hash),
            Err(e) => println!("{}", e),
        }
    }
}
