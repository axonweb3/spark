#[cfg(test)]
mod tests {
    use axon_types::basic::{Byte65, Byte97};
    use bytes::Bytes;
    use ckb_sdk::unlock::ScriptSigner;
    use ckb_types::h256;

    use common::traits::tx_builder::IStakeTxBuilder;
    use common::types::tx_builder::{
        CkbNetwork, DelegateRequirement, FirstStakeInfo, NetworkType, StakeItem, StakeTypeIds,
    };
    use molecule::prelude::Entity;
    use ophelia::PublicKey;
    use ophelia_blst::BlsPublicKey;
    use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;

    use crate::ckb::stake::StakeTxBuilder;
    use crate::ckb::utils::omni::{omni_eth_address, omni_eth_ckb_address, omni_eth_signer};
    use crate::ckb::utils::tx::{gen_script_group, send_tx};

    fn gen_pubkey() -> (Byte65, Byte97) {
        let pub_key =
            hex_decode("ac85bbb40347b6e06ac2dc2da1f75eece029cdc0ed2d456c457d27e288bfbfbcd4c5c19716e9b250134a0e76ce50fa22");
        let bls_public_key: BlsPublicKey = BlsPublicKey::try_from(pub_key.as_ref()).unwrap();
        (
            Byte65::new_unchecked(Bytes::from(pub_key)),
            Byte97::new_unchecked(bls_public_key.to_bytes()),
        )
    }

    #[test]
    fn bls_pub_key() {
        let (pub_key, bls_pub_key) = gen_pubkey();
        println!("pub key len: {:?}", pub_key.as_bytes().len());
        println!("bls pub key len: {}", bls_pub_key.as_bytes().len());
    }

    // #[tokio::test]
    async fn _first_stake_tx() {
        let (l1_pub_key, bls_pub_key) = gen_pubkey();

        _stake_tx(
            StakeItem {
                is_increase:        true,
                amount:             10,
                inauguration_epoch: 3,
            },
            Some(FirstStakeInfo {
                l1_pub_key,
                bls_pub_key,
                delegate: DelegateRequirement {
                    commission_rate:    80,
                    maximum_delegators: 2,
                    threshold:          0,
                },
            }),
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

    async fn _stake_tx(stake_item: StakeItem, first_stake: Option<FirstStakeInfo>) {
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
            first_stake,
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

    fn hex_decode(src: &str) -> Vec<u8> {
        if src.is_empty() {
            return Vec::new();
        }

        let src = if src.starts_with("0x") {
            src.split_at(2).1
        } else {
            src
        };

        let src = src.as_bytes();
        let mut ret = vec![0u8; src.len() / 2];
        faster_hex::hex_decode(src, &mut ret).unwrap();

        ret
    }
}
