use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct ChainState {
    block_number: i64,
    epoch:        i64,
    period:       i64,
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
    pub epoch:        i32,
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
    pub state_amount:    i32,
    pub delegate_amount: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RewardState {
    pub lock_reward_amount:   i32,
    pub unlock_reward_amount: i32,
}
