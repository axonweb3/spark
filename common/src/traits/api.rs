use crate::Result;
use async_trait::async_trait;

use crate::types::{relation_db::transaction::Model, smt::Address};

#[async_trait]
pub trait APIAdapter: Send + Sync {
    async fn get_records_by_address(
        &self,
        addr: Address,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Model>>;

    async fn get_operation_history(
        &self,
        addr: Address,
        operation: u32,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Model>>;

    async fn get_stake_amount_by_epoch(
        &self,
        operation: u32,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Model>>;

    async fn get_top_stake_address(&self, operation: u32) -> Result<Vec<Model>>;

    async fn get_address_state(&self, addr: Address) -> Result<Vec<Model>>;

    async fn get_latest_stake_transactions(
        &self,
        offset: u64,
        page_size: u64,
    ) -> Result<Vec<Model>>;
}
