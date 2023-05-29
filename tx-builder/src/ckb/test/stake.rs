#[cfg(test)]
mod tests {
    use ckb_sdk::unlock::ScriptSigner;
    use ckb_types::h256;

    use common::traits::tx_builder::IStakeTxBuilder;
    use common::types::tx_builder::{
        CkbNetwork, DelegateRequirement, NetworkType, StakeItem, StakeTypeIds,
    };
    use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;

    use crate::ckb::stake::StakeTxBuilder;
    use crate::ckb::utils::omni::{omni_eth_address, omni_eth_ckb_address, omni_eth_signer};
    use crate::ckb::utils::tx::{gen_script_group, send_tx};

    // #[tokio::test]
    async fn _first_stake_tx() {
        _stake_tx(
            StakeItem {
                is_increase:        true,
                amount:             10,
                inauguration_epoch: 3,
            },
            Some(DelegateRequirement::default()),
        )
        .await;
    }

    // #[tokio::test]
    async fn _add_stake_tx() {
        _stake_tx(
            StakeItem {
                is_increase:        true,
                amount:             50,
                inauguration_epoch: 3,
            },
            None,
        )
        .await;
    }

    // #[tokio::test]
    async fn _reedem_stake_tx() {
        _stake_tx(
            StakeItem {
                is_increase:        false,
                amount:             30,
                inauguration_epoch: 3,
            },
            None,
        )
        .await;
    }

    async fn _stake_tx(stake_item: StakeItem, delegate: Option<DelegateRequirement>) {
        let test_staker_key =
            h256!("0x13b08bb054d5dd04013156dced8ba2ce4d8cc5973e10d905a228ea1abc267e62");
        let xudt_args = h256!("0xfdaf95d57c615deaed3d7307d3f649b88d50a51f592a428f3815768e5ae3eab3");
        let checkpoint_type_id =
            h256!("0xfdaf95d57c615deaed3d7307d3f649b88d50a51f592a428f3815768e5ae3eab3");
        let metadata_type_id =
            h256!("0xfdaf95d57c615deaed3d7307d3f649b88d50a51f592a428f3815768e5ae3eab3");
        let ckb_client = CkbRpcClient::new("https://testnet.ckb.dev");

        let staker_eth_addr = omni_eth_address(test_staker_key.clone()).unwrap();
        let staker_ckb_addr =
            omni_eth_ckb_address(&NetworkType::Testnet, test_staker_key.clone()).unwrap();
        println!("staker addr: {}", staker_ckb_addr);

        let mut tx = StakeTxBuilder::new(
            CkbNetwork {
                network_type: NetworkType::Testnet,
                client:       ckb_client.clone(),
            },
            StakeTypeIds {
                metadata_type_id,
                checkpoint_type_id,
                xudt_owner: xudt_args,
            },
            staker_eth_addr,
            1,
            stake_item,
            delegate,
        )
        .build_tx()
        .await
        .unwrap();

        println!("tx: {}", tx);

        let signer = omni_eth_signer(test_staker_key).unwrap();

        let script_groups = gen_script_group(&ckb_client, &tx).await.unwrap();

        for group in script_groups.lock_groups.iter() {
            println!("id: {:?}", group.1.input_indices);
            tx = signer.sign_tx(&tx, group.1).unwrap();
        }

        match send_tx(&ckb_client, &tx.data().into()).await {
            Ok(tx_hash) => println!("tx hash: 0x{}", tx_hash),
            Err(e) => println!("{}", e),
        }
    }
}
