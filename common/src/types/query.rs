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

// 定义 QueryHistory trait
trait QueryAccountHistory {
    fn get_stake_history(
        &self,
        address: &str,
    ) -> Vec<(TxId, Timestamp, Amount, TxHash, TransactionStatus)>;

    fn get_delegate_information(&self, address: &str) -> Vec<(Address, Amount)>;

    fn get_reward_information(
        &self,
        address: &str,
    ) -> Vec<(Address, UnlockAmount, LockAmount, TotalAmount)>;
    // ) -> (Address, UnlockAmount, LockAmount, TotalAmount);

    fn get_reward_history(
        &self,
        address: &str,
    ) -> Vec<(
        TxId,
        EpochNum,
        TotalAmount,
        RewardStatus,
        StakeType,
        Address,
        Amount,
    )>;

    fn get_withdraw_history(
        &self,
        address: &str,
        // ) -> Vec<(TxId, Timestamp, Amount, TransactionStatus)>;
    ) -> Vec<(TxId, Timestamp, Amount, TxHash, TransactionStatus)>;
    // 页面虽然没展示 TxHash 但数据结构是可以返回它的
}

trait QueryAxonStatus {
    fn get_amount_info(&self) -> (TotalStakeAmount, TotalDelegateAmount);

    fn get_chain_state(&self) -> (BlockNum, EpochNum, PeriodNum);

    fn get_top_stake_info(&self) -> (StakeRank, Address, Amount);

    fn get_latest_stake_txs(
        &self,
        stake_type: StakeType,
    ) -> Vec<(Timestamp, Address, Amount, TransactionStatus)>;
}

// 定义 TransactionStatus 枚举
#[derive(Debug)]
enum TransactionStatus {
    Success,
    Pending,
    Failed,
}

#[derive(Debug)]
enum RewardStatus {
    Lock,
    Unlock,
}

#[derive(Debug)]
enum StakeType {
    Stake,
    Delegate,
}
