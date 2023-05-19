#[cfg(test)]
mod tests {
    use ckb_types::h256;

    use common::traits::tx_builder::IInitTxBuilder;
    use common::types::tx_builder::{Checkpoint, Metadata, NetworkType, Scripts};
    use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;

    use crate::ckb::init::InitTxBuilder;
    use crate::ckb::utils::tx::send_tx;

    // #[tokio::test]
    async fn _send_init_tx() {
        let test_key = h256!("0x13b08bb054d5dd04013156dced8ba2ce4d8cc5973e10d905a228ea1abc267e60");
        let ckb_client = CkbRpcClient::new("https://testnet.ckb.dev");

        let tx = InitTxBuilder::new(
            ckb_client.clone(),
            NetworkType::Testnet,
            test_key,
            Scripts {
                selection_lock_code_hash: h256!(
                    "0xcf3b976b52bc1837aa0bbf33d369211c507fe3dc45c4043d44a8139e58da200e"
                ),
            },
            Checkpoint::default(),
            Metadata::default(),
        )
        .build_tx()
        .await
        .unwrap();

        println!("tx: {}\n", tx);

        let tx_hash = send_tx(&ckb_client, &tx.data().into()).await.unwrap();
        println!("tx hash: {}", tx_hash);
    }
}
