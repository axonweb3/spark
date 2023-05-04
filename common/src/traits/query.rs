use async_trait::async_trait;
use crate::{types::{H256, RpcResult, query::NodeStakeHistory}, TempQueryError};


#[async_trait]
pub trait QueryInformation {
    async fn get_stake_history(
        &self,
        address: H256,
    ) -> Result<NodeStakeHistory, TempQueryError>;

    async fn get_delegate_information(&self, address: H256) -> Result<Vec<(H256)>, TempQueryError>;

    async fn get_reward_information<'a>(
        &self,
        address: H256,
    ) -> Result<Vec<(H256)>, TempQueryError>;

    async fn get_reward_history<'a>(
        &self,
        address: H256,
    ) -> Result<Vec<(H256)>, TempQueryError>;

    async fn get_withdraw_history(
        &self,
        address: String,
    ) -> Result<Vec<(H256)>, TempQueryError>;
        
    async fn get_amount_info(&self) -> Result<(H256, H256), TempQueryError>;

    async fn get_top_stake_info(&self) -> Result<(H256, H256), TempQueryError>;

    async fn get_latest_stake_txs(
        &self,
        stake_type: H256,
    ) -> Result<Vec<(H256)>, TempQueryError>;
}

trait QueryAxonStatus {
    fn get_chain_state(&self) -> Result<(H256, H256), TempQueryError>;

}
