use axon_types::{
    delegate::{DelegateAtCellData as ADelegateAtCellData, DelegateInfoDeltas},
    metadata::{MetadataCellData as AMetadataCellData, MetadataList},
    stake::StakeAtCellData as AStakeAtCellData,
    withdraw::{
        WithdrawAtCellData as AWithdrawAtCellData, WithdrawInfo as AWithdrawInfo,
        WithdrawInfos as AWithdrawInfos,
    },
};
use ckb_types::{H160, H256};
use molecule::prelude::{Builder, Entity};

use common::types::tx_builder::*;
use common::utils::convert::*;

#[derive(Clone, Default)]
pub struct StakeGroupInfo {
    pub staker:                   H160,
    pub delegate_infos:           Vec<DelegateInfo>,
    pub delegate_old_epoch_proof: Vec<u8>,
    pub delegate_new_epoch_proof: Vec<u8>,
}

#[derive(Clone, Default)]
pub struct DelegateInfo {
    pub delegator_addr: H160,
    pub amount:         u128,
}

#[derive(Clone, Default)]
pub struct StakerSmtRoot {
    pub staker: H160,
    pub root:   H256,
}

#[derive(Clone, Default)]
pub struct DelegateSmtCellData {
    // pub version:          u8, // useless
    pub metadata_type_id: H256,               // useless
    pub smt_roots:        Vec<StakerSmtRoot>, // smt root of all delegator infos
}

#[derive(Clone, Default)]
pub struct StakeInfo {
    pub addr:   H160,
    pub amount: u128,
}

#[derive(Clone, Default)]
pub struct StakeSmtUpdateInfo {
    pub all_stake_infos: Vec<StakeInfo>,
    pub old_epoch_proof: Vec<u8>,
    pub new_epoch_proof: Vec<u8>,
}

#[derive(Clone, Default)]
pub struct StakeSmtCellData {
    // pub version:          u8, // useless
    pub metadata_type_id: H256, // useless
    pub smt_root:         H256,
}

#[derive(Clone, Default)]
pub struct StakeAtCellData {
    // pub version:          u8, // useless
    // pub l1_address:       H160, // useless
    // pub l2_address:       H160, // useless
    // pub metadata_type_id: H256, // useless
    pub stake_info: StakeItem,
}

impl From<StakeAtCellData> for AStakeAtCellData {
    fn from(value: StakeAtCellData) -> Self {
        AStakeAtCellData::new_builder()
            .delta((&value.stake_info).into())
            .build()
    }
}

#[derive(Clone, Default)]
pub struct DelegateAtCellData {
    // pub version:          u8, // useless
    // pub l1_address:       H160, // useless
    // pub metadata_type_id: H256, // useless
    pub delegator_infos: Vec<DelegateItem>,
}

impl From<DelegateAtCellData> for ADelegateAtCellData {
    fn from(value: DelegateAtCellData) -> Self {
        let infos = DelegateInfoDeltas::new_builder()
            .extend(value.delegator_infos.iter().map(Into::into))
            .build();
        ADelegateAtCellData::new_builder()
            // .version(value.version.into()) // useless
            // .l1_address(Identity::from_slice(value.l1_address.as_bytes()).unwrap()) // useless
            // .metadata_type_id(to_byte32(&value.metadata_type_id)) // useless
            .delegator_infos(infos)
            .build()
    }
}

#[derive(Clone, Default)]
pub struct WithdrawAtCellData {
    // pub version:          u8, // useless
    // pub metadata_type_id: H256, // useless
    pub withdraw_infos: Vec<WithdrawInfo>,
}

impl From<WithdrawAtCellData> for AWithdrawAtCellData {
    fn from(value: WithdrawAtCellData) -> Self {
        let infos: AWithdrawInfos = AWithdrawInfos::new_builder()
            .extend(value.withdraw_infos.into_iter().map(Into::into))
            .build();
        AWithdrawAtCellData::new_builder()
            // .version(value.version.into()) // useless
            // .metadata_type_id(to_byte32(&value.metadata_type_id)) // useless
            .withdraw_infos(infos)
            .build()
    }
}

#[derive(Clone, Default)]
pub struct WithdrawInfo {
    pub amount: u128,
    pub epoch:  u64,
}

impl From<WithdrawInfo> for AWithdrawInfo {
    fn from(value: WithdrawInfo) -> Self {
        AWithdrawInfo::new_builder()
            .amount(to_uint128(value.amount))
            .epoch(to_uint64(value.epoch))
            .build()
    }
}

#[derive(Clone, Default)]
pub struct MetadataCellData {
    // pub version:                u8, // useless
    pub epoch:                  u64,
    pub propose_count_smt_root: H256,
    pub type_ids:               TypeIds,
    pub metadata:               Vec<Metadata>,
}

impl From<MetadataCellData> for AMetadataCellData {
    fn from(v: MetadataCellData) -> Self {
        AMetadataCellData::new_builder()
            // .version(v.version.into()) // useless
            .epoch(to_uint64(v.epoch))
            .propose_count_smt_root(to_byte32(&v.propose_count_smt_root))
            .type_ids((v.type_ids).into())
            .metadata({
                let mut list = MetadataList::new_builder();
                for m in v.metadata.iter() {
                    list = list.push(m.into());
                }
                list.build()
            })
            .build()
    }
}
