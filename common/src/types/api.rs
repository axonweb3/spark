use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct ChainState {
    block_number: u64,
    epoch:        u64,
    period:       u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum HistoryEvent {
    Add,
    Redeem,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum OperationType {
    Stake,
    Delegate,
    Reward,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum LockStatusType {
    Lock,
    Unlock,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StakeAmount {
    pub epoch:        u32,
    pub amount:       String,
    pub operate_type: OperationType,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AddressAmount {
    pub address: String,
    pub amount:  String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StakeState {
    pub state_amount:    u32,
    pub delegate_amount: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RewardState {
    pub lock_reward_amount:   u32,
    pub unlock_reward_amount: u32,
}
