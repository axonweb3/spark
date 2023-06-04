use std::sync::Arc;

use crate::jsonrpc::OperationRpcServer;
use common::{
    traits::api::APIAdapter,
    types::{api::OperationType, Transaction, H256},
};
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
    async fn set_stake_rate(
        &self,
        _address: H256,
        _stake_rate: u64,
        _delegate_rate: u64,
    ) -> RpcResult<String> {
        let _ = self.adapter;
        unimplemented!()
    }

    async fn stake(&self, _address: H256, _amount: u64) -> RpcResult<String> {
        let _ = self.adapter;
        unimplemented!()
    }

    async fn unstake(&self, _address: H256, _amount: u64) -> RpcResult<String> {
        let _ = self.adapter;
        unimplemented!()
    }

    async fn delegate(&self, _address: H256, _amount: u64) -> RpcResult<String> {
        unimplemented!()
    }

    async fn undelegate(&self, _address: H256, _amount: u64) -> RpcResult<String> {
        unimplemented!()
    }

    async fn withdraw_stake(
        &self,
        _address: H256,
        _withdraw_type: OperationType,
    ) -> RpcResult<String> {
        // withdraw_type: stake | delegate
        unimplemented!()
    }

    async fn withdraw_rewards(&self, _address: H256) -> RpcResult<String> {
        unimplemented!()
    }

    async fn send_transaction(&self, _tx: Transaction) -> RpcResult<H256> {
        unimplemented!()
    }
}
