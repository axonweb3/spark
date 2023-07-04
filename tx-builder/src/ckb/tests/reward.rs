#[cfg(test)]
mod tests {
    use std::{path::PathBuf, vec};

    use ckb_sdk::types::{ScriptGroup, ScriptGroupType};
    use ckb_sdk::unlock::ScriptSigner;
    use ckb_types::h256;

    use common::traits::tx_builder::IRewardTxBuilder;
    use common::types::tx_builder::{Epoch, RewardInfo, RewardTypeIds};
    use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;
    use storage::smt::SmtManager;

    use crate::ckb::helper::ckb::{OmniEth, Tx};
    use crate::ckb::reward::RewardTxBuilder;

    // #[tokio::test]
    async fn _reward_tx(current_epoch: Epoch) {
        let test_staker_key =
            h256!("0x13b08bb054d5dd04013156dced8ba2ce4d8cc5973e10d905a228ea1abc267e62");
        let xudt_args = h256!("0xfdaf95d57c615deaed3d7307d3f649b88d50a51f592a428f3815768e5ae3eab3");
        let selection_type_id =
            h256!("0xfdaf95d57c615deaed3d7307d3f649b88d50a51f592a428f3815768e5ae3eab3");
        let checkpoint_type_id =
            h256!("0xfdaf95d57c615deaed3d7307d3f649b88d50a51f592a428f3815768e5ae3eab3");
        let metadata_type_id =
            h256!("0xfdaf95d57c615deaed3d7307d3f649b88d50a51f592a428f3815768e5ae3eab3");
        let reward_smt_type_id =
            h256!("0xfdaf95d57c615deaed3d7307d3f649b88d50a51f592a428f3815768e5ae3eab3");
        let stake_smt_type_id =
            h256!("0xfdaf95d57c615deaed3d7307d3f649b88d50a51f592a428f3815768e5ae3eab3");
        let delegate_smt_type_id =
            h256!("0xfdaf95d57c615deaed3d7307d3f649b88d50a51f592a428f3815768e5ae3eab3");
        let ckb_client = CkbRpcClient::new("https://testnet.ckb.dev");

        let omni_eth = OmniEth::new(test_staker_key.clone());
        let staker_eth_addr = omni_eth.address().unwrap();
        let staker_ckb_addr = omni_eth.ckb_address().unwrap();
        println!("staker addr: {}", staker_ckb_addr);

        let staker_lock = OmniEth::lock(&staker_eth_addr);

        let path = PathBuf::from("./free-space/smt");
        let smt = SmtManager::new(path);

        let tx = RewardTxBuilder::new(
            &ckb_client,
            RewardTypeIds {
                selection_type_id,
                metadata_type_id,
                checkpoint_type_id,
                reward_smt_type_id,
                stake_smt_type_id,
                delegate_smt_type_id,
                xudt_owner: xudt_args,
            },
            smt,
            RewardInfo {
                base_reward:               10000,
                half_reward_cycle:         100,
                theoretical_propose_count: 10,
                epoch_count:               10,
            },
            staker_eth_addr,
            current_epoch,
        )
        .build_tx()
        .await
        .unwrap();

        println!("tx: {}", tx);

        let signer = omni_eth.signer().unwrap();
        let tx = signer
            .sign_tx(&tx, &ScriptGroup {
                script:         staker_lock.clone(),
                group_type:     ScriptGroupType::Lock,
                input_indices:  vec![1],
                output_indices: vec![0],
            })
            .unwrap();
        let tx = signer
            .sign_tx(&tx, &ScriptGroup {
                script:         staker_lock,
                group_type:     ScriptGroupType::Lock,
                input_indices:  vec![2],
                output_indices: vec![0],
            })
            .unwrap();

        match Tx::new(&ckb_client, tx).send().await {
            Ok(tx_hash) => println!("tx hash: 0x{}", tx_hash),
            Err(e) => println!("{}", e),
        }
    }
}
