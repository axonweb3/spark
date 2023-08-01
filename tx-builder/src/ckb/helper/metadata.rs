use anyhow::Result;
use bytes::Bytes;
use ckb_types::packed::{CellDep, OutPoint, Script};
use ckb_types::prelude::{Builder, Entity, Pack};
use ckb_types::H256;

use common::traits::ckb_rpc_client::CkbRpc;
use common::types::axon_types::metadata::MetadataCellData;
use common::types::ckb_rpc_client::Cell;
use common::types::tx_builder::{NetworkType, RewardMeta};
use common::utils::convert::{to_u128, to_u16, to_u32, to_u64};

use crate::ckb::define::scripts::*;
use crate::ckb::helper::ckb::cell_collector::get_cell_by_type;
use crate::ckb::helper::unique_cell_dep;
use crate::ckb::NETWORK_TYPE;
use crate::{cell_dep, out_point, script};

pub struct Metadata;

impl Metadata {
    pub fn type_(args: &H256) -> Script {
        let args = Bytes::from(args.as_bytes().to_vec());
        match **NETWORK_TYPE.load() {
            NetworkType::Mainnet => script!(
                &METADATA_TYPE_MAINNET.code_hash,
                METADATA_TYPE_MAINNET.hash_type,
                args
            ),
            NetworkType::Testnet => script!(
                &METADATA_TYPE_TESTNET.code_hash,
                METADATA_TYPE_TESTNET.hash_type,
                args
            ),
            NetworkType::Devnet => script!(
                &METADATA_TYPE_DEVNET.code_hash,
                METADATA_TYPE_DEVNET.hash_type,
                args
            ),
        }
    }

    pub fn type_dep() -> CellDep {
        match **NETWORK_TYPE.load() {
            NetworkType::Mainnet => cell_dep!(
                &METADATA_TYPE_MAINNET.tx_hash,
                METADATA_TYPE_MAINNET.index,
                METADATA_TYPE_MAINNET.dep_type
            ),
            NetworkType::Testnet => cell_dep!(
                &METADATA_TYPE_TESTNET.tx_hash,
                METADATA_TYPE_TESTNET.index,
                METADATA_TYPE_TESTNET.dep_type
            ),
            NetworkType::Devnet => cell_dep!(
                &METADATA_TYPE_DEVNET.tx_hash,
                METADATA_TYPE_DEVNET.index,
                METADATA_TYPE_DEVNET.dep_type
            ),
        }
    }

    pub async fn cell_dep(ckb_rpc: &impl CkbRpc, type_id: &H256) -> Result<CellDep> {
        unique_cell_dep(ckb_rpc, Self::type_(type_id)).await
    }

    pub async fn get_cell(ckb_rpc: &impl CkbRpc, metadata_type: Script) -> Result<Cell> {
        get_cell_by_type(ckb_rpc, metadata_type).await
    }

    pub fn parse_quorum(metadata_cell_data: &MetadataCellData) -> u16 {
        to_u16(&metadata_cell_data.metadata().get(1).unwrap().quorum())
    }

    pub fn parse_reward_meta(metadata_cell_data: &MetadataCellData) -> RewardMeta {
        RewardMeta {
            base_reward:           to_u128(&metadata_cell_data.base_reward()),
            half_reward_cycle:     to_u64(&metadata_cell_data.half_epoch()),
            propose_minimum_rate:  metadata_cell_data.propose_minimum_rate().into(),
            propose_discount_rate: metadata_cell_data.propose_discount_rate().into(),
        }
    }

    pub fn calc_minimum_propose_count(metadata: &MetadataCellData) -> u64 {
        let propose_minimum_rate: u8 = metadata.propose_minimum_rate().into();
        let metadata = metadata.metadata().get(0).unwrap();
        let epoch_block_count = to_u32(&metadata.epoch_len()) * to_u32(&metadata.period_len());
        let validator_num = metadata.validators().len();
        (epoch_block_count * propose_minimum_rate as u32 / validator_num as u32 / 100).into()
    }
}
