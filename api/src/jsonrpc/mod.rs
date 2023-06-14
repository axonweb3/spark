pub mod axon;
pub mod operation;
pub mod query;
use crate::error::ApiError;
use crate::jsonrpc::operation::OperationRpc;
use crate::jsonrpc::query::{AxonStatusRpc, StatusRpcModule};
use common::types::api::{
    AddressAmount, ChainState, HistoryEvent, OperationType, RewardHistory, RewardState,
    StakeAmount, StakeHistory, StakeRate, StakeState, StakeTransaction,
};
use common::types::smt::Address;
use common::types::Transaction;
use common::{traits::api::APIAdapter, types::H256};
use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::server::ServerBuilder;
use std::{net::SocketAddr, sync::Arc};

#[rpc(server)]
pub trait AccountHistoryRpc {
    #[method(name = "getStakeRate")]
    async fn get_stake_rate(&self, addr: Address) -> RpcResult<StakeRate>;

    #[method(name = "getStakeState")]
    async fn get_stake_state(&self, addr: Address) -> RpcResult<StakeState>;

    #[method(name = "getRewardState")]
    async fn get_reward_state(&self, addr: Address) -> RpcResult<RewardState>;

    #[method(name = "getStakeHistory")]
    async fn get_stake_history(
        &self,
        addr: Address,
        page_number: u64,
        page_size: u64,
        enent: HistoryEvent,
        operation_type: OperationType,
    ) -> RpcResult<Vec<StakeHistory>>;

    #[method(name = "getRewardHistory")]
    async fn get_reward_history(
        &self,
        addr: Address,
        page_number: u64,
        page_size: u64,
    ) -> RpcResult<RewardHistory>;

    #[method(name = "getStakeAmountByEpoch")]
    async fn get_stake_amount_by_epoch(
        &self,
        operation_type: OperationType,
        page_number: u64,
        page_size: u64,
    ) -> RpcResult<Vec<StakeAmount>>;

    #[method(name = "getTopStakeAddress")]
    async fn get_top_stake_address(
        &self,
        page_number: u64,
        page_size: u64,
    ) -> RpcResult<Vec<AddressAmount>>;

    #[method(name = "getLatestStakeTransactions")]
    async fn get_latest_stake_transactions(
        &self,
        page_number: u64,
        page_size: u64,
    ) -> RpcResult<Vec<StakeTransaction>>;
}

#[rpc(server)]
pub trait AxonStatusRpc {
    #[method(name = "getChainState")]
    async fn get_chain_state(&self) -> RpcResult<ChainState>;
}

#[rpc(server)]
pub trait OperationRpc {
    #[method(name = "setStakeRate")]
    async fn set_stake_rate(
        &self,
        address: H256,
        stake_rate: u64,
        delegate_rate: u64,
    ) -> RpcResult<String>;

    #[method(name = "stake")]
    async fn stake(&self, address: H256, amount: u64) -> RpcResult<String>;

    #[method(name = "unstake")]
    async fn unstake(&self, address: H256, amount: u64) -> RpcResult<String>;

    #[method(name = "delegate")]
    async fn delegate(&self, address: H256, amount: u64) -> RpcResult<String>;

    #[method(name = "undelegate")]
    async fn undelegate(&self, address: H256, amount: u64) -> RpcResult<String>;

    #[method(name = "withdrawStake")]
    async fn withdraw_stake(
        &self,
        address: H256,
        withdraw_type: OperationType,
    ) -> RpcResult<String>;

    #[method(name = "withdrawRewards")]
    async fn withdraw_rewards(&self, address: H256) -> RpcResult<String>;

    #[method(name = "sendTransaction")]
    async fn send_transaction(&self, tx: Transaction) -> RpcResult<H256>;
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
