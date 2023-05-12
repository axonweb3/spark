use std::sync::Arc;

use crate::{
    error::ApiError,
    jsonrpc::{AccountHistoryRpcServer, AxonStatusRpcServer},
};
use common::{
    traits::api::APIAdapter,
    types::{
        api::{
            AddressAmount, ChainState, HistoryEvent, LockStatusType, OperationType, RewardState,
            StakeAmount, StakeState,
        },
        relation_db::transaction::Model,
        smt::Address,
    },
};
use jsonrpsee::core::{async_trait, RpcResult};

pub struct StatusRpcModule<Adapter> {
    adapter: Arc<Adapter>,
}

impl<Adapter: APIAdapter> StatusRpcModule<Adapter> {
    #[allow(dead_code)]
    pub fn new(adapter: Arc<Adapter>) -> Self {
        Self { adapter }
    }
}

#[async_trait]
impl<Adapter: APIAdapter + 'static> AccountHistoryRpcServer for StatusRpcModule<Adapter> {
    async fn get_stake_history(
        &self,
        addr: Address,
        page_number: u64,
        page_size: u64,
        enent: HistoryEvent,
        history_type: OperationType,
    ) -> RpcResult<Vec<Model>> {
        let offset = (page_number - 1) * page_size;
        let history_type = history_type as u32;
        let res = self
            .adapter
            .get_operation_history(addr, history_type, offset, page_size)
            .await
            .map_err(|e| ApiError::Adapter(e.to_string()))?;
        let event_type = enent as u32;
        let adds: Vec<Model> = res
            .iter()
            .filter(|m| m.event == event_type)
            .cloned()
            .collect();
        Ok(adds)
    }

    async fn get_reward_history(
        &self,
        addr: Address,
        page_number: u64,
        page_size: u64,
        lock_type: LockStatusType,
    ) -> RpcResult<Vec<Model>> {
        let offset = (page_number - 1) * page_size;
        let reward_type = OperationType::Reward as u32;
        let res = self
            .adapter
            .get_operation_history(addr, reward_type, offset, page_size)
            .await
            .map_err(|e| ApiError::Adapter(e.to_string()))?;
        let lock_type = lock_type as u32;
        let adds: Vec<Model> = res
            .iter()
            .filter(|m| m.status == lock_type)
            .cloned()
            .collect();
        Ok(adds)
    }

    async fn get_stake_amount_by_epoch(
        &self,
        operation_type: OperationType,
        page_number: u64,
        page_size: u64,
    ) -> RpcResult<Vec<StakeAmount>> {
        let offset = (page_number - 1) * page_size;
        let res = self
            .adapter
            .get_stake_amount_by_epoch(operation_type as u32, offset, page_size)
            .await
            .map_err(|e| ApiError::Adapter(e.to_string()))?;
        let res: Vec<StakeAmount> = res
            .into_iter()
            .map(|model| {
                let operation_type = match model.operation {
                    1 => OperationType::Stake,
                    2 => OperationType::Delegate,
                    3 => OperationType::Reward,
                    _ => panic!("Invalid operation type"),
                };
                StakeAmount {
                    epoch:        model.epoch,
                    amount:       model.amount,
                    operate_type: OperationType::from(operation_type),
                }
            })
            .collect();
        Ok(res)
    }

    async fn get_top_stake_address(&self, page_size: u64) -> RpcResult<Vec<AddressAmount>> {
        let res = self
            .adapter
            .get_top_stake_address(OperationType::Stake as u32)
            .await
            .map_err(|e| ApiError::Adapter(e.to_string()))?;
        let res: Vec<AddressAmount> = res
            .iter()
            .take(page_size as usize)
            .map(|m| AddressAmount {
                address: m.address.clone(),
                amount:  m.amount.clone(),
            })
            .collect();
        Ok(res)
    }

    async fn get_stake_state(&self, addr: Address) -> RpcResult<StakeState> {
        let res = self
            .adapter
            .get_address_state(addr)
            .await
            .map_err(|e| ApiError::Adapter(e.to_string()))?;
        let (state_amount, delegate_amount) = res.iter().fold((0, 0), |res, model| {
            if model.operation == OperationType::Stake as u32 {
                (res.0 + model.amount.parse::<u32>().unwrap(), res.1)
            } else if model.operation == OperationType::Delegate as u32 {
                (res.0, res.1 + model.amount.parse::<u32>().unwrap())
            } else {
                res
            }
        });
        let res = StakeState {
            state_amount,
            delegate_amount,
        };
        Ok(res)
    }

    async fn get_reward_state(&self, addr: Address) -> RpcResult<RewardState> {
        let res = self
            .adapter
            .get_address_state(addr)
            .await
            .map_err(|e| ApiError::Adapter(e.to_string()))?;
        let (lock_reward_amount, unlock_reward_amount) = res.iter().fold((0, 0), |res, model| {
            if model.operation == OperationType::Stake as u32 {
                (res.0 + model.epoch, res.1)
            } else if model.operation == OperationType::Delegate as u32 {
                (res.0, res.1 + model.epoch)
            } else {
                res
            }
        });
        let res = RewardState {
            lock_reward_amount,
            unlock_reward_amount,
        };
        Ok(res)
    }
}

pub struct AxonStatusRpc<Adapter> {
    adapter: Arc<Adapter>,
}

impl<Adapter: APIAdapter> AxonStatusRpc<Adapter> {
    #[allow(dead_code)]
    pub fn new(adapter: Arc<Adapter>) -> Self {
        Self { adapter }
    }
}

#[async_trait]
impl<Adapter: APIAdapter + 'static> AxonStatusRpcServer for AxonStatusRpc<Adapter> {
    async fn get_chain_state(&self) -> RpcResult<ChainState> {
        let res = ChainState::default();
        let _ = self.adapter;
        // ChainState::default();
        Ok(res)
    }
}
