use anyhow::Result;
use async_trait::async_trait;

use crate::types::{
    relation_db::transaction::{self, Model},
    smt::Address,
};

#[async_trait]
pub trait TransactionStorage {
    async fn insert(&mut self, tx_record: transaction::ActiveModel) -> Result<()>;

    async fn get_records_by_address(
        &self,
        addr: Address,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Model>>;
}
