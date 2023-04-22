use ethereum_types::H256;
use sea_orm::sea_query::error::Error;

use crate::types::smt::{Address, Amount};

#[derive(PartialEq, Eq)]
pub enum TransactionStatus {
    Success,
    Pending,
    Fail,
}

#[derive(PartialEq, Eq)]
pub enum OperationType {
    StakeAdd,
    StakeRedeem,
    DelegateAdd,
    DelegateRedeem,
    Withdraw,
}

pub struct TransactionRecord {
    address:   Address,
    timestamp: u8,
    operation: OperationType,
    tx_hash:   H256,
    amount:    Amount,
    status:    TransactionStatus,
}

trait TransactionHistory {
    fn insert(&mut self, tx_record: TransactionRecord) -> Result<(), Error>;

    fn get_stake_records_by_address(&self, addr: Address) -> Result<Vec<TransactionRecord>, Error>;

    fn get_delegate_records_by_address(
        &self,
        addr: Address,
    ) -> Result<Vec<TransactionRecord>, Error>;

    fn get_withdraw_records_by_address(
        &self,
        addr: Address,
    ) -> Result<Vec<TransactionRecord>, Error>;
}
