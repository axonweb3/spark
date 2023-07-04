use std::sync::Arc;

use crate::{
    error::ApiError,
    jsonrpc::{AccountHistoryRpcServer, AxonStatusRpcServer},
};
use common::{
    traits::api::APIAdapter,
    types::{
        api::{
            AddressAmount, ChainState, HistoryEvent, HistoryTransactions, OperationStatus,
            OperationType, RewardFrom, RewardHistory, RewardState, StakeAmount, StakeHistory,
            StakeRate, StakeState, StakeTransaction,
        },
        smt::Address,
    },
};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::{error::INVALID_PARAMS_CODE, ErrorObjectOwned},
};

pub struct StatusRpcModule<Adapter> {
    adapter: Arc<Adapter>,
}

impl<Adapter: APIAdapter> StatusRpcModule<Adapter> {
    pub fn new(adapter: Arc<Adapter>) -> Self {
        Self { adapter }
    }
}

#[async_trait]
impl<Adapter: APIAdapter + 'static> AccountHistoryRpcServer for StatusRpcModule<Adapter> {
    async fn get_stake_rate(&self, addr: Address) -> RpcResult<StakeRate> {
        let res = self
            .adapter
            .get_records_by_address(addr, 0, 1)
            .await
            .map_err(|e| ApiError::Adapter(e.to_string()))?;

        res.get(0)
            .map(|s| StakeRate {
                address:       addr.to_string(),
                stake_rate:    s.stake_rate.clone(),
                delegate_rate: s.delegate_rate.clone(),
            })
            .ok_or(ErrorObjectOwned::owned(
                INVALID_PARAMS_CODE,
                "wrong number of arguments".to_string(),
                None::<()>,
            ))
    }

    async fn get_stake_state(&self, addr: Address) -> RpcResult<StakeState> {
        let res = self
            .adapter
            .get_address_state(addr)
            .await
            .map_err(|e| ApiError::Adapter(e.to_string()))?;
        let (stake_amount, amount, delegate_amount, withdrawable_amount) =
            res.iter().fold((0, 0, 0, 0), |res, model| {
                if model.operation == OperationType::Stake as u32 {
                    (
                        res.0 + model.total_amount,
                        res.1,
                        res.2 + model.delegate_amount,
                        res.3 + model.withdrawable_amount,
                    )
                } else if model.operation == OperationType::Delegate as u32 {
                    (
                        res.0,
                        res.1 + model.total_amount,
                        res.2 + model.delegate_amount,
                        res.3 + model.withdrawable_amount,
                    )
                } else {
                    res
                }
            });
        let res = StakeState {
            total_amount: amount,
            stake_amount,
            delegate_amount,
            withdrawable_amount,
        };
        Ok(res)
    }

    async fn get_reward_state(&self, addr: Address) -> RpcResult<RewardState> {
        let res = self
            .adapter
            .get_records_by_address(addr, 0, 1)
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
            lock_amount:   lock_reward_amount,
            unlock_amount: unlock_reward_amount,
        };
        Ok(res)
    }

    async fn get_stake_history(
        &self,
        addr: Address,
        page_number: u64,
        page_size: u64,
        event: HistoryEvent,
        history_type: OperationType,
    ) -> RpcResult<Vec<StakeHistory>> {
        let offset = (page_number - 1) * page_size;
        let history_type = history_type as u32;
        let res = self
            .adapter
            .get_operation_history(addr, history_type, offset, page_size)
            .await
            .map_err(|e| ApiError::Adapter(e.to_string()))?;
        let event_type = event as u32;

        let txs = res.iter().filter(|m| m.event == event_type).cloned().fold(
            Vec::new(),
            |mut acc, model| {
                let transaction = HistoryTransactions {
                    hash:      model.tx_hash.parse().unwrap(),
                    status:    OperationStatus::from(model.status),
                    timestamp: model.timestamp as u64,
                };
                acc.push(transaction);
                acc
            },
        );

        let ret = res
            .iter()
            .filter(|m| m.event == event_type)
            .cloned()
            .map(|model| StakeHistory {
                id: addr.to_string(),
                amount: model.total_amount,
                event,
                status: OperationStatus::from(model.status),
                transactions: txs.clone(),
            })
            .collect();
        Ok(ret)
    }

    async fn get_reward_history(
        &self,
        addr: Address,
        page_number: u64,
        page_size: u64,
    ) -> RpcResult<RewardHistory> {
        let offset = (page_number - 1) * page_size;
        let reward_type = OperationType::Reward as u32;
        let res = self
            .adapter
            .get_operation_history(addr, reward_type, offset, page_size)
            .await
            .map_err(|e| ApiError::Adapter(e.to_string()))?;
        res.get(0)
            .map(|s| RewardHistory {
                epoch:  s.epoch,
                amount: s.total_amount,
                locked: s.status != 0,
                from:   RewardFrom {
                    reward_type: s.operation.into(),
                    address:     addr,
                    amount:      s.total_amount as u64,
                },
            })
            .ok_or(ErrorObjectOwned::owned(
                INVALID_PARAMS_CODE,
                "wrong number of arguments".to_string(),
                None::<()>,
            ))
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
            .map(|model| StakeAmount {
                epoch:  model.epoch,
                amount: model.total_amount.to_string(),
            })
            .collect();
        Ok(res)
    }

    async fn get_top_stake_address(
        &self,
        page_number: u64,
        page_size: u64,
    ) -> RpcResult<Vec<AddressAmount>> {
        let total_num = page_number * page_size;
        let res = self
            .adapter
            .get_top_stake_address(OperationType::Stake as u32)
            .await
            .map_err(|e| ApiError::Adapter(e.to_string()))?;
        let res: Vec<AddressAmount> = res
            .iter()
            .take(total_num as usize)
            .map(|m| AddressAmount {
                address: m.address.clone(),
                amount:  m.total_amount.to_string(),
            })
            .collect();
        Ok(res)
    }

    async fn get_latest_stake_transactions(
        &self,
        page_number: u64,
        page_size: u64,
    ) -> RpcResult<Vec<StakeTransaction>> {
        let offset = (page_number - 1) * page_size;
        let res = self
            .adapter
            .get_latest_stake_transactions(offset, page_size)
            .await
            .map_err(|e| ApiError::Adapter(e.to_string()))?;

        let stake_transactions = res
            .iter()
            .map(|model| StakeTransaction {
                timestamp: model.timestamp as u64,
                hash:      model.tx_hash.parse().unwrap(),
                amount:    model.total_amount as u64,
                status:    OperationStatus::from(model.status),
            })
            .collect();

        Ok(stake_transactions)
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
