pub mod checkpoint;
mod define;
pub mod delegate;
pub mod delegate_smt;
pub mod helper;
pub mod init;
pub mod metadata;
pub mod mint;
pub mod reward;
pub mod stake;
pub mod stake_smt;
mod tests;
pub mod withdraw;

use arc_swap::ArcSwap;
use common::types::tx_builder::NetworkType;

lazy_static::lazy_static! {
    pub static ref NETWORK_TYPE: ArcSwap<NetworkType> = ArcSwap::from_pointee(NetworkType::Testnet);
}
