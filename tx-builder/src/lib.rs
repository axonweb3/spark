pub mod axon;
pub mod ckb;

use common::types::tx_builder::NetworkType;
use std::sync::Arc;

pub fn set_network_type(network_type: NetworkType) {
    (*ckb::NETWORK_TYPE).swap(Arc::new(network_type));
}
