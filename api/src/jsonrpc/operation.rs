use std::sync::Arc;

use crate::jsonrpc::OperationRpcServer;
use common::{traits::api::APIAdapter, types::H256};
use jsonrpsee::core::{async_trait, RpcResult};

pub struct OperationRpc<Adapter> {
    adapter: Arc<Adapter>,
}

impl<Adapter: APIAdapter> OperationRpc<Adapter> {
    pub fn new(adapter: Arc<Adapter>) -> Self {
        Self { adapter }
    }
}

#[async_trait]
impl<Adapter: APIAdapter + 'static> OperationRpcServer for OperationRpc<Adapter> {
    async fn add_stake(&self, _address: H256, _amount: u64) -> RpcResult<String> {
        let _ = self.adapter;
        unimplemented!()
    }

    async fn redeem_stake(&self, _address: H256, _amount: u64) -> RpcResult<String> {
        unimplemented!()
    }

    async fn add_delegate(&self, _address: H256, _amount: u64) -> RpcResult<String> {
        unimplemented!()
    }

    async fn redeem_delegate(&self, _address: H256, _amount: u64) -> RpcResult<String> {
        unimplemented!()
    }

    async fn withdraw(&self) -> RpcResult<Vec<H256>> {
        unimplemented!()
    }

    async fn unlock_reward(&self) -> RpcResult<Vec<H256>> {
        unimplemented!()
    }

    async fn send_transaction(&self) -> RpcResult<Vec<H256>> {
        unimplemented!()
    }
}
