use async_trait::async_trait;
use crate::types::{H256, RpcResult, query::NodeStakeHistory};


#[async_trait]
pub trait QueryInformation {
    async fn get_stake_history(
        &self,
        address: H256,
    ) -> RpcResult<NodeStakeHistory>;

    async fn get_delegate_information(&self, address: H256) -> Vec<(H256)>;

    async fn get_reward_information<'a>(
        &self,
        address: H256,
    ) -> Vec<(H256)>;

    async fn get_reward_history<'a>(
        &self,
        address: H256,
    ) -> Vec<(H256)>;

    async fn get_withdraw_history(
        &self,
        address: String,
    ) -> Vec<(H256)>;
        
    async fn get_amount_info(&self) -> (H256, H256);

    async fn get_top_stake_info(&self) -> (H256, H256);

    async fn get_latest_stake_txs(
        &self,
        stake_type: H256,
    ) -> Vec<(H256)>;
}

trait QueryAxonStatus {
    fn get_chain_state(&self) -> (H256, H256);

}
