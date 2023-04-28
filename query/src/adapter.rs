use crate::QueryError;
use async_trait::async_trait;
use common::types::H256;
use common::{
    traits::query::QueryInformation,
    types::{query::NodeStakeHistory, RpcResult},
};
pub struct DefaultQueryAdapter {}

#[async_trait]
impl QueryInformation for DefaultQueryAdapter {
    async fn get_stake_history(&self, address: H256) -> RpcResult<NodeStakeHistory> {
        let res = NodeStakeHistory::default();
        Ok(res)
    }

    async fn get_delegate_information(&self, address: H256) -> Vec<(H256)> {
        vec![H256::default()]
    }

    async fn get_reward_information<'a>(&self, address: H256) -> Vec<(H256)> {
        vec![H256::default()]
    }

    async fn get_reward_history<'a>(&self, address: H256) -> Vec<(H256)> {
        vec![H256::default()]
    }

    async fn get_withdraw_history(&self, address: String) -> Vec<(H256)> {
        vec![H256::default()]
    }

    async fn get_amount_info(&self) -> (H256, H256) {
        (H256::default(), H256::default())
    }

    async fn get_top_stake_info(&self) -> (H256, H256) {
        (H256::default(), H256::default())
    }

    async fn get_latest_stake_txs(&self, stake_type: H256) -> Vec<(H256)> {
        vec![H256::default()]
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_tx_address() {}
}
