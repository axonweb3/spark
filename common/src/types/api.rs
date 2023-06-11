use crate::types::H160;
use ckb_types::H256;
use serde::{Deserialize, Serialize};

use crate::types::axon_rpc_client::{Header, Metadata};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct ChainState {
    pub epoch:        u64,
    pub block_number: u64,
}

impl ChainState {
    pub fn new(h: Header, m: Metadata) -> Self {
        ChainState {
            block_number: h.number,
            epoch:        m.version.end.saturating_sub(m.version.start),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum HistoryEvent {
    Add,
    Redeem,
}

impl From<u32> for HistoryEvent {
    fn from(value: u32) -> Self {
        match value {
            0 => HistoryEvent::Add,
            1 => HistoryEvent::Redeem,
            _ => panic!("Invalid value for HistoryEvent"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum OperationType {
    Stake,
    Delegate,
    Reward,
}

impl From<u32> for OperationType {
    fn from(value: u32) -> Self {
        match value {
            0 => OperationType::Stake,
            1 => OperationType::Delegate,
            2 => OperationType::Reward,
            _ => panic!("Invalid value for OperationType"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum OperationStatus {
    Success,
    Pending,
    Failed,
}

impl From<u32> for OperationStatus {
    fn from(value: u32) -> Self {
        match value {
            0 => OperationStatus::Success,
            1 => OperationStatus::Pending,
            2 => OperationStatus::Failed,
            _ => panic!("Invalid value for OperationStatus"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum LockStatusType {
    Lock,
    Unlock,
}

impl From<u32> for LockStatusType {
    fn from(value: u32) -> Self {
        match value {
            0 => LockStatusType::Lock,
            1 => LockStatusType::Unlock,
            _ => panic!("Invalid value for LockStatusType"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StakeAmount {
    pub epoch:  u32,
    pub amount: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StakeRate {
    pub address:       String,
    pub stake_rate:    String,
    pub delegate_rate: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AddressAmount {
    pub address: String,
    pub amount:  String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StakeState {
    pub total_amount:        u32,
    pub stake_amount:        u32,
    pub delegate_amount:     u32,
    pub withdrawable_amount: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StakeHistory {
    pub id:           String,
    pub amount:       u32,
    pub event:        HistoryEvent,
    pub status:       OperationStatus,
    pub transactions: Vec<HistoryTransactions>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HistoryTransactions {
    pub hash:      H256,
    pub status:    OperationStatus,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RewardState {
    pub lock_amount:   u32,
    pub unlock_amount: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RewardHistory {
    pub epoch:  u32,
    pub amount: u32,
    pub locked: bool,
    pub from:   RewardFrom,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RewardFrom {
    pub reward_type: OperationType,
    pub address:     H160,
    pub amount:      u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StakeTransaction {
    pub timestamp: u64,
    pub hash:      H256,
    pub amount:    u64,
    pub status:    OperationStatus,
}
