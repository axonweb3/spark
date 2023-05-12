pub mod operation;
pub mod query;
use crate::error::ApiError;
use crate::jsonrpc::operation::OperationRpc;
use crate::jsonrpc::query::{AxonStatusRpc, StatusRpcModule};
use common::types::api::{
    AddressAmount, ChainState, HistoryEvent, LockStatusType, OperationType, RewardState,
    StakeAmount, StakeState,
};
use common::types::relation_db::transaction::Model;
use common::types::smt::Address;
use common::{traits::api::APIAdapter, types::H256};
use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::server::ServerBuilder;
use std::{net::SocketAddr, sync::Arc};

#[rpc(server)]
pub trait AccountHistoryRpc {
    /// Sends signed transaction, returning its hash.
    #[method(name = "getStakeHistory")]
    async fn get_stake_history(
        &self,
        addr: Address,
        page_number: u64,
        page_size: u64,
        enent: HistoryEvent,
        operation_type: OperationType,
    ) -> RpcResult<Vec<Model>>;

    #[method(name = "getRewardHistory")]
    async fn get_reward_history(
        &self,
        addr: Address,
        page_number: u64,
        page_size: u64,
        lock_type: LockStatusType,
    ) -> RpcResult<Vec<Model>>;

    #[method(name = "getStakeAmountByEpoch")]
    async fn get_stake_amount_by_epoch(
        &self,
        operation_type: OperationType,
        page_number: u64,
        page_size: u64,
    ) -> RpcResult<Vec<StakeAmount>>;

    #[method(name = "getTopStakeAddress")]
    async fn get_top_stake_address(&self, page_size: u64) -> RpcResult<Vec<AddressAmount>>;

    #[method(name = "getStakeState")]
    async fn get_stake_state(&self, addr: Address) -> RpcResult<StakeState>;

    #[method(name = "getRewardState")]
    async fn get_reward_state(&self, addr: Address) -> RpcResult<RewardState>;
}

#[rpc(server)]
pub trait AxonStatusRpc {
    #[method(name = "get_chain_state")]
    async fn get_chain_state(&self) -> RpcResult<ChainState>;
}

#[rpc(server)]
pub trait OperationRpc {
    #[method(name = "addStake")]
    async fn add_stake(&self, address: H256, amount: u64) -> RpcResult<String>;

    #[method(name = "redeemStake")]
    async fn redeem_stake(&self, address: H256, amount: u64) -> RpcResult<String>;

    #[method(name = "addDelegate")]
    async fn add_delegate(&self, address: H256, amount: u64) -> RpcResult<String>;

    #[method(name = "redeemDelegate")]
    async fn redeem_delegate(&self, address: H256, amount: u64) -> RpcResult<String>;

    #[method(name = "withdraw")]
    async fn withdraw(&self) -> RpcResult<Vec<H256>>;

    #[method(name = "unlockReward")]
    async fn unlock_reward(&self) -> RpcResult<Vec<H256>>;

    #[method(name = "sendTransaction")]
    async fn send_transaction(&self) -> RpcResult<Vec<H256>>;
}

#[allow(dead_code)]
pub async fn mock_server<Adapter: APIAdapter + 'static>(
    adapter: Arc<Adapter>,
) -> Result<SocketAddr, ApiError> {
    let mut module = StatusRpcModule::new(Arc::clone(&adapter)).into_rpc();
    let axon_rpc = AxonStatusRpc::new(Arc::clone(&adapter)).into_rpc();
    let op_rpc = OperationRpc::new(adapter).into_rpc();
    module.merge(axon_rpc).unwrap();
    module.merge(op_rpc).unwrap();
    let server = ServerBuilder::new()
        .http_only()
        .build("127.0.0.1:0".parse::<SocketAddr>().unwrap())
        .await
        .map_err(|e| ApiError::HttpServer(e.to_string()))?;
    println!("addr: {:?}", server.local_addr().unwrap());
    // module.register_method("a_method", |_, _| "lo").unwrap();

    let addr = server.local_addr().unwrap();
    let handle = server.start(module).unwrap();

    tokio::spawn(handle.stopped());

    Ok(addr)
}
