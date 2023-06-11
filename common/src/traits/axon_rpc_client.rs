use async_trait::async_trait;

use crate::types::{api::ChainState, axon_rpc_client::LatestCheckPointInfo, ckb_rpc_client::Cell};
use anyhow::Result;

#[async_trait]
pub trait SubmitProcess {
    fn is_closed(&self) -> bool;
    // if false return, it means this cell process should be shutdown
    async fn notify_axon(&mut self, cell: &Cell) -> bool;
}

#[async_trait]
pub trait AxonRpc: Send + Sync {
    async fn get_checkpoint_info(&self) -> Result<LatestCheckPointInfo>;
}

#[async_trait]
pub trait AxonWsRpc: Send + Sync {
    async fn sub_axon_header(&self) -> Result<ChainState>;
}
