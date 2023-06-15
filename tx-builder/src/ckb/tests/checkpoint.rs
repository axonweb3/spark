#[cfg(test)]
mod tests {

    use ckb_types::h256;
    use common::{
        traits::tx_builder::ICheckpointTxBuilder,
        types::{
            axon_rpc_client::mock_latest_check_point_info,
            tx_builder::{Checkpoint, CheckpointTypeIds, CkbNetwork, NetworkType},
        },
        utils::convert::to_ckb_h256,
    };
    use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;

    use crate::ckb::{checkpoint::CheckpointTxBuilder, utils::tx::send_tx};

    // #[tokio::test]
    async fn _checkpoints_tx() {
        let test_kicker_key =
            h256!("0x13b08bb054d5dd04013156dced8ba2ce4d8cc5973e10d905a228ea1abc267e62");
        let checkpoint_type_id =
            h256!("0xfe18e5fde2ca0d863fc9888aed7e3d667249d719542d1dd78aa77de0938c2a83");
        let metadata_type_id =
            h256!("0x30bdedc605cdb0b80f7f328c803d6059f0ad7bdeb0ccb8f44019502ac03b68a2");

        let ckb_client = CkbRpcClient::new("https://testnet.ckb.dev");

        let mock_axon_checkpoint_info = mock_latest_check_point_info();

        let mock_checkpoint = Checkpoint {
            epoch:               2,
            period:              2,
            state_root:          to_ckb_h256(&mock_axon_checkpoint_info.state_root),
            latest_block_height: mock_axon_checkpoint_info.latest_block_height,
            latest_block_hash:   to_ckb_h256(&mock_axon_checkpoint_info.latest_block_hash),
            timestamp:           mock_axon_checkpoint_info.timestamp,
            proof:               (&mock_axon_checkpoint_info.proof).into(),
            propose_count:       mock_axon_checkpoint_info
                .propose_count
                .iter()
                .map(|propose| propose.into())
                .collect(),
        };

        let ckb = CkbNetwork {
            network_type: NetworkType::Testnet,
            client:       ckb_client.clone(),
        };

        let tx = CheckpointTxBuilder::new(
            test_kicker_key.clone(),
            ckb,
            CheckpointTypeIds {
                metadata_type_id,
                checkpoint_type_id,
            },
            mock_checkpoint,
        )
        .await
        .build_tx()
        .await
        .unwrap();

        println!("tx: {}", tx);

        match send_tx(&ckb_client, &tx.data().into()).await {
            Ok(tx_hash) => println!("tx hash: 0x{}", tx_hash),
            Err(e) => println!("{}", e),
        }
    }
}
