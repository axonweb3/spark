use anyhow::Result;
use async_trait::async_trait;
use common::traits::ckb_rpc_client::CkbSubscriptionRpc;
use std::path::PathBuf;
use std::sync::{atomic::AtomicPtr, Arc};

use ckb_jsonrpc_types::BlockNumber;
use common::types::ckb_rpc_client::RpcSearchKey;

use crate::axon_client::RpcSubmit;
use crate::ckb_client::{
    cell_process::CellProcess,
    ckb_rpc_client::CkbRpcClient,
    state_handle::GlobalState,
    types::{ScanTip, ScanTipInner, State},
};

pub struct CkbSubscriptionClient {
    cell_handles: Arc<dashmap::DashMap<RpcSearchKey, tokio::task::JoinHandle<()>>>,
    state:        State,
    client:       CkbRpcClient,
}

impl CkbSubscriptionClient {
    pub fn new(ckb_uri: &str, path: PathBuf) -> Self {
        let client = CkbRpcClient::new(ckb_uri);
        let mut global = GlobalState::new(path);
        let state = global.state.clone();

        let cell_handles = global.spawn_cells(client.clone());
        let _global_handle = tokio::spawn(async move { global.run().await });

        Self {
            cell_handles,
            state,
            client,
        }
    }

    async fn register(&self, search_key: RpcSearchKey, start: BlockNumber) -> Result<bool> {
        if self.state.cell_states.contains_key(&search_key) {
            return Ok(false);
        }
        let indexer_tip = self.client.get_indexer_tip().await?;

        if indexer_tip.block_number > start {
            let scan_tip = ScanTip(Arc::new(ScanTipInner(AtomicPtr::new(Box::into_raw(
                Box::new(start),
            )))));

            self.state
                .cell_states
                .insert(search_key.clone(), scan_tip.clone());

            let mut cell_process =
                CellProcess::new(search_key.clone(), scan_tip, self.client.clone(), RpcSubmit);

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
}

#[async_trait]
impl CkbSubscriptionRpc for CkbSubscriptionClient {
    async fn register(&self, search_key: RpcSearchKey, start: BlockNumber) -> Result<bool> {
        self.register(search_key, start).await
    }

    async fn delete(&self, search_key: RpcSearchKey) -> Result<bool> {
        self.delete(search_key).await
    }
}
