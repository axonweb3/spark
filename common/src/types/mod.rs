pub mod api;
pub mod query;
pub mod smt;
pub mod tx_builder;
use jsonrpsee::{core::Error};


pub use ethereum_types::{
    BigEndianHash, Bloom, Public, Secret, Signature, H128, H160, H256, H512, H520, H64, U128,
    U256, U512, U64,
};

pub type RpcResult<T> = Result<T, Error>;

