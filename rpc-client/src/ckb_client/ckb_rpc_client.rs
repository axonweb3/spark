use anyhow::Result;
use async_trait::async_trait;
use ckb_jsonrpc_types::{JsonBytes, Uint32};
use common::{
    traits::ckb_rpc_client::CkbRpc,
    types::ckb_rpc_client::{Cell, IndexerTip, Order, Pagination, SearchKey},
};
use reqwest::{Client, Url};

use std::{
    future::Future,
    io,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use crate::error::RpcError;

macro_rules! jsonrpc {
    ($method:expr, $self:ident, $return:ty$(, $params:ident$(,)?)*) => {{
        let old = $self.id.fetch_add(1, Ordering::AcqRel);
        let data = format!(
            r#"{{"id": {}, "jsonrpc": "2.0", "method": "{}", "params": {}}}"#,
            old,
            $method,
            serde_json::to_value(($($params,)*)).unwrap()
        );

        let req_json: serde_json::Value = serde_json::from_str(&data).unwrap();

        let c = $self.raw.post($self.ckb_uri.clone()).json(&req_json);
        async {
            let resp = c
                .send()
                .await.map_err(|e| RpcError::ConnectionAborted(io::Error::new(io::ErrorKind::ConnectionAborted, format!("{:?}", e))))?;
            let output = resp
                .json::<jsonrpc_core::response::Output>()
                .await.map_err(|e| RpcError::InvalidData(io::Error::new(io::ErrorKind::InvalidData, format!("{:?}", e))))?;

            match output {
                jsonrpc_core::response::Output::Success(success) => {
                    Ok(serde_json::from_value::<$return>(success.result).unwrap())
                }
                jsonrpc_core::response::Output::Failure(e) => {
                    Err(RpcError::InvalidData(io::Error::new(io::ErrorKind::InvalidData, format!("{:?}", e))).into())
                }
            }
        }
    }}
}

// Default implementation of ckb Rpc client
#[derive(Clone)]
pub struct CkbRpcClient {
    raw:     Client,
    ckb_uri: Url,
    id:      Arc<AtomicU64>,
}

impl CkbRpcClient {
    pub fn new(ckb_uri: &str) -> Self {
        let ckb_uri = Url::parse(ckb_uri).expect("ckb uri, e.g. \"http://127.0.0.1:8114\"");

        CkbRpcClient {
            raw: Client::new(),
            ckb_uri,
            id: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn new_with_client(ckb_uri: &str, raw: Client) -> Self {
        let ckb_uri = Url::parse(ckb_uri).expect("ckb uri, e.g. \"http://127.0.0.1:8114\"");

        CkbRpcClient {
            raw,
            ckb_uri,
            id: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn get_cells(
        &self,
        search_key: SearchKey,
        order: Order,
        limit: Uint32,
        after: Option<JsonBytes>,
    ) -> impl Future<Output = Result<Pagination<Cell>>> {
        jsonrpc!(
            "get_cells",
            self,
            Pagination<Cell>,
            search_key,
            order,
            limit,
            after
        )
    }

    pub fn get_indexer_tip(&self) -> impl Future<Output = Result<IndexerTip>> {
        jsonrpc!("get_indexer_tip", self, IndexerTip)
    }
}

#[async_trait]
impl CkbRpc for CkbRpcClient {
    async fn get_cells(
        &self,
        search_key: SearchKey,
        order: Order,
        limit: Uint32,
        after: Option<JsonBytes>,
    ) -> Result<Pagination<Cell>> {
        self.get_cells(search_key, order, limit, after).await
    }

    async fn get_indexer_tip(&self) -> Result<IndexerTip> {
        self.get_indexer_tip().await
    }
}
