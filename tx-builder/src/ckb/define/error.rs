use ckb_types::H160 as CH160;
use ethereum_types::H160;
use thiserror::Error;

use common::types::tx_builder::{Amount, Epoch};

pub type CkbTxResult<T> = std::result::Result<T, CkbTxErr>;

#[derive(Error, Debug)]
pub enum CkbTxErr {
    #[error("Missing information for the first stake")]
    FirstStake,

    #[error("Invalid inaugration epoch, expected: {expected:?}, found: {found:?}")]
    InaugurationEpoch { expected: Epoch, found: Epoch },

    #[error(
        "The stake/delegate amount is too large, wallet amount: {0}, stake/delegate amount: {1}"
    )]
    ExceedWalletAmount(Amount, Amount),

    #[error("The stake/delegate amount is too large, total elect amount: {total_amount:?}, stake/delegate amount: {new_amount:?}")]
    ExceedTotalAmount {
        total_amount: Amount,
        new_amount:   Amount,
    },

    #[error("Invalid is_increase: {0}")]
    Increase(bool),

    #[error("Lack of capacity: {inputs_capacity:?} < {outputs_capacity:?}")]
    InsufficientCapacity {
        inputs_capacity:  u64,
        outputs_capacity: u64,
    },

    #[error(
        "The minted amount is too large, minted amount: {total_mint:?}, max supply: {max_supply:?}"
    )]
    ExceedMaxSupply {
        max_supply: Amount,
        total_mint: Amount,
    },

    #[error("Cell not found: {0}")]
    CellNotFound(String),

    #[error("Deserialize bls pub key error")]
    Deserialize,

    #[error("User's reward epoch not found")]
    RewardEpochNotFound,

    #[error("The minimum value of the current epoch should be 2")]
    EpochTooSmall,

    #[error("Starting epoch is less than ending epoch. start epoch: {0}, end epoch: {1}")]
    RewardEpoch(u64, u64),

    #[error("Stake amount not found in stack SMT. epoch: {0}, staker: {1}")]
    StakeAmountNotFound(u64, H160),

    #[error(
        "Not right checkpoint occassion, latest epoch {current_epoch:?} and period {current_period:?}, recorded epoch {recorded_epoch:?} and period {recorded_period:?} is not meet the condition"
    )]
    NotCheckpointOccasion {
        current_epoch:   u64,
        current_period:  u32,
        recorded_epoch:  u64,
        recorded_period: u32,
    },

    #[error("There should be only one smt cell for the tx, found: {0}")]
    SmtCellNum(usize),

    #[error(
        "Invalid delegate, staker: {0}, delegator: {1}, redeem amount: {2}, total amount: {3}"
    )]
    RedeemDelegate(CH160, CH160, Amount, Amount),
}
