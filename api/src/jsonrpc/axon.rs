use std::sync::Arc;

use crate::jsonrpc::AxonStatusRpcServer;
use common::{
    traits::{api::APIAdapter, async_trait},
    types::api::ChainState,
};
use jsonrpsee::core::RpcResult;

pub struct AxonStatusRpc<Adapter> {
    _adapter: Arc<Adapter>,
}

impl<Adapter: APIAdapter> AxonStatusRpc<Adapter> {
    // #[warn(dead_code)]
    // pub fn new(_adapter: Arc<Adapter>) -> Self {
    //     Self { _adapter }
    // }
}

#[async_trait]
impl<Adapter: APIAdapter + 'static> AxonStatusRpcServer for AxonStatusRpc<Adapter> {
    async fn get_chain_state(&self) -> RpcResult<ChainState> {
        unimplemented!()
    }
}
