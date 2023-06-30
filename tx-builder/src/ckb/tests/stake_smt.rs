#[cfg(test)]
mod tests {
    use ckb_types::h256;

    use common::traits::tx_builder::IStakeSmtTxBuilder;
    use common::types::tx_builder::StakeSmtTypeIds;
    use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;
    use storage::smt::SmtManager;

    use crate::ckb::stake_smt::StakeSmtTxBuilder;

    static _ROCKSDB_PATH: &str = "./free-space/smt";

    // #[tokio::test]
    async fn _stake_smt_tx() {
        let smt_storage = SmtManager::new(_ROCKSDB_PATH);
        let test_staker_key =
            h256!("0x13b08bb054d5dd04013156dced8ba2ce4d8cc5973e10d905a228ea1abc267e62");
        let xudt_args = h256!("0xfdaf95d57c615deaed3d7307d3f649b88d50a51f592a428f3815768e5ae3eab3");
        let checkpoint_type_id =
            h256!("0xfdaf95d57c615deaed3d7307d3f649b88d50a51f592a428f3815768e5ae3eab3");
        let metadata_type_id =
            h256!("0xfdaf95d57c615deaed3d7307d3f649b88d50a51f592a428f3815768e5ae3eab3");
        let stake_smt_type_id =
            h256!("0xfdaf95d57c615deaed3d7307d3f649b88d50a51f592a428f3815768e5ae3eab3");
        let ckb_client = CkbRpcClient::new("https://testnet.ckb.dev");

        let (tx, _) = StakeSmtTxBuilder::new(
            &ckb_client,
            test_staker_key,
            1,
            StakeSmtTypeIds {
                metadata_type_id,
                stake_smt_type_id,
                checkpoint_type_id,
                xudt_owner: xudt_args,
            },
            10,
            vec![],
            smt_storage,
        )
        .build_tx()
        .await
        .unwrap();

        println!("tx: {}", tx);
    }
}
