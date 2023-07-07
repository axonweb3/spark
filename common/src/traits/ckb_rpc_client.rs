use anyhow::Result;
use async_trait::async_trait;
use ckb_types::H256;

use crate::types::ckb_rpc_client::{Cell, IndexerTip, Order, Pagination, RpcSearchKey, SearchKey};
use crate::types::{
    BlockNumber, CellWithStatus, JsonBytes, OutPoint, OutputsValidator, Transaction,
    TransactionWithStatusResponse, Uint32,
};

#[async_trait]
pub trait CkbRpc: Send + Sync + Clone {
    async fn get_cells(
        &self,
        search_key: SearchKey,
        order: Order,
        limit: Uint32,
        after: Option<JsonBytes>,
    ) -> Result<Pagination<Cell>>;

    async fn get_live_cell(&self, out_point: OutPoint, with_data: bool) -> Result<CellWithStatus>;

    // ckb indexer `get_indexer_tip`
    async fn get_indexer_tip(&self) -> Result<IndexerTip>;

    // Pool
    async fn send_transaction(
        &self,
        tx: &Transaction,
        outputs_validator: Option<OutputsValidator>,
    ) -> Result<H256>;

    // Chain
    async fn get_transaction(&self, hash: H256) -> Result<Option<TransactionWithStatusResponse>>;
}

#[async_trait]
pub trait CkbSubscriptionRpc {
    async fn register(&self, search_key: RpcSearchKey, start: BlockNumber) -> Result<bool>;

    async fn delete(&self, search_key: RpcSearchKey) -> Result<bool>;
}
