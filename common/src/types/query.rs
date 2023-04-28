use crate::types::H256;

#[derive(Debug, Default)]
pub struct NodeStakeHistory {
    Txs: Vec<TransactionInfo>,
    Account: u64,
}

#[derive(Debug)]
pub struct NodeDelegateHistory {

}

#[derive(Debug)]
pub struct AxonStakeHistory {

}

#[derive(Debug, Default)]
pub struct TransactionInfo {
    TxId: H256,
    Timestamp: u64,
    Amount: u64,
    TxHash: String,
    Address: String,
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
