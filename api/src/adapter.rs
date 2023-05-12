use common::traits::{
    api::APIAdapter,
    async_trait,
    query::TransactionStorage,
    smt::{DelegateSmtStorage, RewardSmtStorage, StakeSmtStorage},
};
use common::types::{relation_db::transaction::Model, smt::Address};
use common::Result;
use std::sync::Arc;

#[derive(Clone)]
pub struct DefaultAPIAdapter<T, S> {
    relation_storage: Arc<T>,
    _smt_storage:     Arc<S>,
}

impl<T, S> DefaultAPIAdapter<T, S>
where
    T: TransactionStorage + 'static,
    S: StakeSmtStorage + DelegateSmtStorage + RewardSmtStorage + 'static,
{
    pub fn new(relation_storage: Arc<T>, smt_storage: Arc<S>) -> Self {
        Self {
            relation_storage,
            _smt_storage: smt_storage,
        }
    }
}

#[async_trait]
impl<T, S> APIAdapter for DefaultAPIAdapter<T, S>
where
    T: TransactionStorage + Sync + Send + 'static,
    S: StakeSmtStorage + DelegateSmtStorage + RewardSmtStorage + Sync + Send + 'static,
{
    async fn get_records_by_address(
        &self,
        addr: Address,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Model>> {
        self.relation_storage
            .get_records_by_address(addr, offset, limit)
            .await
    }

    async fn get_operation_history(
        &self,
        addr: Address,
        operation: u32,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Model>> {
        self.relation_storage
            .get_operation_history(addr, operation, offset, limit)
            .await
    }

    async fn get_stake_amount_by_epoch(
        &self,
        operation: u32,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Model>> {
        self.relation_storage
            .get_stake_amount_by_epoch(operation, offset, limit)
            .await
    }

    async fn get_top_stake_address(&self, operation: u32) -> Result<Vec<Model>> {
        self.relation_storage.get_top_stake_address(operation).await
    }

    async fn get_address_state(&self, addr: Address) -> Result<Vec<Model>> {
        self.relation_storage.get_address_state(addr).await
    }
}
