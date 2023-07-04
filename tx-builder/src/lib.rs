pub mod axon;
pub mod ckb;

use ckb_types::H256;
use common::types::tx_builder::NetworkType;
use std::sync::Arc;

pub fn init_static_variables(
    network_type: NetworkType,
    metadata_type_id: H256,
    checkpoint_type_id: H256,
) {
    (*ckb::NETWORK_TYPE).swap(Arc::new(network_type));
    (*ckb::METADATA_TYPE_ID).swap(Arc::new(metadata_type_id));
    (*ckb::CHECKPOINT_TYPE_ID).swap(Arc::new(checkpoint_type_id));
}
