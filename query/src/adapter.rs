
use crate::QueryError;
use common::types::query::QueryInformation;
use async_trait::async_trait;
pub struct DefaultQueryAdapter {

}

#[async_trait]
impl QueryInformation for DefaultQueryAdapter {
    async fn get_stake_history<'a>(
        &self,
        address: &'a str,
    ) -> Vec<(TxId, Timestamp, Amount, TxHash, TransactionStatus)> where &'a str: 'async_trait{}

    async fn get_delegate_information(&self, address: String) -> Vec<(Address, Amount)>{}

    async fn get_reward_information(
        &self,
        address: String,
    ) -> Vec<(Address, UnlockAmount, LockAmount, TotalAmount)>{}

    async fn get_reward_history(
        &self,
        address: String,
    ) -> Vec<(
        TxId,
        EpochNum,
        TotalAmount,
        RewardStatus,
        StakeType,
        Address,
        Amount,
    )>{}

    fn get_withdraw_history(
        &self,
        address: String,
    ) -> Vec<(TxId, Timestamp, Amount, TxHash, TransactionStatus)>{}
        
    async fn get_amount_info(&self) -> (TotalStakeAmount, TotalDelegateAmount){}

    async fn get_top_stake_info(&self) -> (StakeRank, Address, Amount){}

    async fn get_latest_stake_txs(
        &self,
        stake_type: StakeType,
    ) -> Vec<(Timestamp, Address, Amount, TransactionStatus)>{}
}

#[cfg(test)]
mod tests {


    #[test]
    fn test_tx_address() {
        
    }
}