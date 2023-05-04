use common::TempQueryError;
use async_trait::async_trait;
use common::types::H256;
use common::{
    traits::query::QueryInformation,
    types::{query::NodeStakeHistory, RpcResult},
};
pub struct DefaultQueryAdapter {}

#[async_trait]
impl QueryInformation for DefaultQueryAdapter {
    // async fn get_stake_history(&self, address: H256) -> RpcResult<NodeStakeHistory> {
    async fn get_stake_history(&self, address: H256) -> Result<NodeStakeHistory, TempQueryError> {
        let res = NodeStakeHistory::default();
        Ok(res)
    }

    async fn get_delegate_information(&self, address: H256) -> Result<Vec<(H256)>, TempQueryError> {
        Ok(vec![H256::default()])
    }

    async fn get_reward_information<'a>(&self, address: H256) -> Result<Vec<(H256)>, TempQueryError> {
        Ok(vec![H256::default()])
    }

    async fn get_reward_history<'a>(&self, address: H256) -> Result<Vec<(H256)>, TempQueryError> {
        Ok(vec![H256::default()])
    }

    async fn get_withdraw_history(&self, address: String) -> Result<Vec<(H256)>, TempQueryError> {
        Ok(vec![H256::default()])
    }

    async fn get_amount_info(&self) -> Result<(H256, H256), TempQueryError> {
        Ok((H256::default(), H256::default()))
    }

    async fn get_top_stake_info(&self) -> Result<(H256, H256), TempQueryError> {
        Ok((H256::default(), H256::default()))
    }

    async fn get_latest_stake_txs(&self, stake_type: H256) -> Result<Vec<(H256)>, TempQueryError> {
        Ok(vec![H256::default()])
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_tx_address() {}
}
