use async_trait::async_trait;
use migration::DbErr;
use transaction_entity::transaction::{self, Model};

use crate::types::smt::Address;

#[async_trait]
pub trait TransactionStorage {
    async fn insert(&mut self, tx_record: transaction::ActiveModel) -> Result<(), DbErr>;

    async fn get_records_by_address(
        &self,
        addr: Address,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Model>, DbErr>;
}
