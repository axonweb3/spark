use thiserror::Error;

use common::types::tx_builder::{Amount, Epoch};

pub type CkbTxResult<T> = std::result::Result<T, CkbTxErr>;

#[derive(Error, Debug)]
pub enum CkbTxErr {
    #[error("Missing delegate information for the first stake")]
    Delegate,

    #[error("Invalid inaugration epoch, expected: {expected:?}, found: {found:?}")]
    InaugurationEpoch { expected: Epoch, found: Epoch },

    #[error("The stake/delegate amount is too large, wallet amount: {wallet_amount:?}, stake/delegate amount: {amount:?}")]
    ExceedWalletAmount {
        wallet_amount: Amount,
        amount:        Amount,
    },

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
}
