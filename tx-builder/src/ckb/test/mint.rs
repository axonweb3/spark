#[cfg(test)]
mod tests {
    use ckb_types::h256;

    use common::traits::tx_builder::IMintTxBuilder;
    use common::types::tx_builder::NetworkType;
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
            h256!("0x22e63905cc0dd6daecfbe4c7293cce6153225f441b979188ee0b9c9191f2f72b");
        let issue_type_id =
            h256!("0x7191fd278c25a0f2de508c03c02ebbfe1b769614dd00eedf116fde2b37c7dea4");
        let ckb_client = CkbRpcClient::new("https://testnet.ckb.dev");

        let tx = MintTxBuilder::new(
            CkbRpcClient::new("https://testnet.ckb.dev"),
            NetworkType::Testnet,
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
