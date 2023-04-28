use async_trait::async_trait;
type TxId = String;
type Timestamp = u64;
type Amount = u64;
type TxHash = String;
type Address = String;
type UnlockAmount = u64;
type LockAmount = u64;
type TotalAmount = u64;
type EpochNum = u64;

type TotalStakeAmount = u64;
type TotalDelegateAmount = u64;
type PeriodNum = u64;
type BlockNum = u64;
type StakeRank = u64;
type TokenAmount = u64;

#[async_trait]
pub trait QueryInformation {
    async fn get_stake_history<'a>(
        &self,
        address: &'a str,
    ) -> Vec<(TxId, Timestamp, Amount, TxHash, TransactionStatus)>where &'a str: 'async_trait;

    async fn get_delegate_information<'a>(&self, address: String) -> Vec<(Address, Amount)>;

    async fn get_reward_information<'a>(
        &self,
        address: String,
    ) -> Vec<(Address, UnlockAmount, LockAmount, TotalAmount)>;

    async fn get_reward_history<'a>(
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
    )>;

    async fn get_withdraw_history(
        &self,
        address: String,
    ) -> Vec<(TxId, Timestamp, Amount, TxHash, TransactionStatus)>;
        
    async fn get_amount_info(&self) -> (TotalStakeAmount, TotalDelegateAmount);

    async fn get_top_stake_info(&self) -> (StakeRank, Address, Amount);

    async fn get_latest_stake_txs(
        &self,
        stake_type: StakeType,
    ) -> Vec<(Timestamp, Address, Amount, TransactionStatus)>;
}

trait QueryAxonStatus {
    fn get_chain_state(&self) -> (BlockNum, EpochNum, PeriodNum);

}

#[derive(Debug)]
pub enum TransactionStatus {
    Success,
    Pending,
    Failed,
}

#[derive(Debug)]
pub enum RewardStatus {
    Lock,
    Unlock,
}

#[derive(Debug)]
pub enum StakeType {
    Stake,
    Delegate,
}
