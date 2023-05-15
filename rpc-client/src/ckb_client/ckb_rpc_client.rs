use crate::{axon_client::RpcSubmit, error::RpcError};
use anyhow::Result;
use async_trait::async_trait;
use ckb_jsonrpc_types::{BlockNumber, JsonBytes, Uint32};
use common::{
    traits::ckb_rpc_client::CkbRpc,
    types::ckb_rpc_client::{Cell, IndexerTip, Order, Pagination, RpcSearchKey, SearchKey},
};
use reqwest::{Client, Url};

use std::{
    future::Future,
    io,
    path::PathBuf,
    sync::{
        atomic::{AtomicPtr, AtomicU64, Ordering},
        Arc,
    },
};

use crate::ckb_client::{
    cell_process::CellProcess,
    state_handle::GlobalState,
    types::{ScanTip, ScanTipInner, State},
};

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
pub struct CkbClient {
    raw:              Client,
    ckb_uri:          Url,
    id:               Arc<AtomicU64>,
    pub cell_handles: Arc<dashmap::DashMap<RpcSearchKey, tokio::task::JoinHandle<()>>>,
    pub state:        State,
}

impl CkbClient {
    pub fn new(ckb_uri: &str, path: PathBuf) -> Self {
        let ckb_uri = Url::parse(ckb_uri).expect("ckb uri, e.g. \"http://127.0.0.1:8114\"");
        let mut global = GlobalState::new(path);
        let state = global.state.clone();

        let mut client = CkbClient {
            raw: Client::new(),
            ckb_uri,
            id: Arc::new(AtomicU64::new(0)),
            cell_handles: Arc::new(dashmap::DashMap::with_capacity(state.cell_states.len())),
            state,
        };
        let cell_handles = global.spawn_cells(client.clone());
        let _global_handle = tokio::spawn(async move { global.run().await });

        client.cell_handles = cell_handles;

        client
    }

    async fn register(&self, search_key: RpcSearchKey, start: BlockNumber) -> Result<bool> {
        if self.state.cell_states.contains_key(&search_key) {
            return Ok(false);
        }
        let indexer_tip = self.get_indexer_tip().await?;

        if indexer_tip.block_number > start {
            let scan_tip = ScanTip(Arc::new(ScanTipInner(AtomicPtr::new(Box::into_raw(
                Box::new(start),
            )))));

            self.state
                .cell_states
                .insert(search_key.clone(), scan_tip.clone());

            let mut cell_process =
                CellProcess::new(search_key.clone(), scan_tip, self.clone(), RpcSubmit);

            let handle = tokio::spawn(async move {
                cell_process.run().await;
            });

            self.cell_handles.insert(search_key, handle);
            return Ok(true);
        }

        Ok(false)
    }

    async fn delete(&self, search_key: RpcSearchKey) -> Result<bool> {
        if self.state.cell_states.remove(&search_key).is_some() {
            if let Some(handle) = self.cell_handles.get(&search_key) {
                handle.abort();
                return Ok(true);
            }
        }
        Ok(false)
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
impl CkbRpc for CkbClient {
    async fn register(&self, search_key: RpcSearchKey, start: BlockNumber) -> Result<bool> {
        self.register(search_key, start).await
    }

    async fn delete(&self, search_key: RpcSearchKey) -> Result<bool> {
        self.delete(search_key).await
    }

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
