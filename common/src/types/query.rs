
// type TxId = String;
// type Timestamp = u64;
// type Amount = u64;
// type TxHash = String;
// type Address = String;
// type UnlockAmount = u64;
// type LockAmount = u64;
// type TotalAmount = u64;
// type EpochNum = u64;

// type TotalStakeAmount = u64;
// type TotalDelegateAmount = u64;
// type PeriodNum = u64;
// type BlockNum = u64;
// type StakeRank = u64;
// type TokenAmount = u64;

#[derive(Debug)]
pub struct KickerStatus {
    
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
