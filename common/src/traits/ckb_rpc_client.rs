use anyhow::Result;
use async_trait::async_trait;
use ckb_jsonrpc_types::{JsonBytes, Uint32};

use ckb_jsonrpc_types::BlockNumber;

use crate::types::ckb_rpc_client::{Cell, IndexerTip, Order, Pagination, RpcSearchKey, SearchKey};

#[async_trait]
pub trait CkbRpc {
    async fn get_cells(
        &self,
        search_key: SearchKey,
        order: Order,
        limit: Uint32,
        after: Option<JsonBytes>,
    ) -> Result<Pagination<Cell>>;

    // ckb indexer `get_indexer_tip`
    async fn get_indexer_tip(&self) -> Result<IndexerTip>;
}

#[async_trait]
pub trait CkbSubscriptionRpc {
    async fn register(&self, search_key: RpcSearchKey, start: BlockNumber) -> Result<bool>;

    async fn delete(&self, search_key: RpcSearchKey) -> Result<bool>;
}
