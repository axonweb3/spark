use common::types::{H160, H256};

use crate::QueryError;

#[derive(Debug)]
enum TransactionStatus {
    Success,
    Pending,
    Failed,
}

impl Default for TransactionStatus {
    fn default() -> Self {
        TransactionStatus::Failed
    }
}

#[derive(Debug)]
enum RewardStatus {
    Lock,
    Unlock,
}

#[derive(Debug)]
enum OperationType {
    StakeAdd,
    StakeRedeem,
    DelegateAdd,
    DelegateRedeem,
    Withdraw,
}

impl Default for OperationType {
    fn default() -> Self {
        OperationType::Withdraw
    }
}

#[derive(Default)]
struct Record {
    Address: H160,
    Timestamp: i8,
    Operation: OperationType,
    TxHash: H256,
    Amount: u128,
    Status: TransactionStatus,
}

trait SqlDB {
    fn insert(&mut self, data: Record) -> Result<(), QueryError>;
    fn get_stake_records_by_address(&self, addr: H160) -> Vec<Record>;
    fn get_delegate_records_by_address(&self, addr: H160) -> Vec<Record>;
    fn get_withdraw_records_by_address(&self, addr: H160) -> Vec<Record>;
    // no delete and update apis.
}

pub struct DbMock {}

impl SqlDB for DbMock {
    fn insert(&mut self, data: Record) -> Result<(), QueryError> {
        Ok(())
    }
    fn get_stake_records_by_address(&self, addr: H160) -> Vec<Record> {
        vec![Record::default()]
    }
    fn get_delegate_records_by_address(&self, addr: H160) -> Vec<Record> {
        vec![Record::default()]
    }
    fn get_withdraw_records_by_address(&self, addr: H160) -> Vec<Record> {
        vec![Record::default()]
    }
}
