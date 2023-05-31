use std::collections::HashMap;

use axon_types::{
    basic::{Byte20, Byte32, Byte65, Byte97, Identity},
    checkpoint::{CheckpointCellData, ProposeCount as AProposeCount, ProposeCounts},
    delegate::{DelegateAtCellData as ADelegateAtCellData, DelegateInfoDelta, DelegateInfoDeltas},
    metadata::{
        MetaTypeIds, Metadata as AMetadata, MetadataCellData as AMetadataCellData, MetadataList,
        Validator as AValidator, ValidatorList,
    },
    withdraw::{
        WithdrawAtCellData as AWithdrawAtCellData, WithdrawInfo as AWithdrawInfo,
        WithdrawInfos as AWithdrawInfos,
    },
};
use ckb_types::{H160, H256};
use molecule::prelude::{Builder, Byte, Entity};

use crate::traits::ckb_rpc_client::CkbRpc;
use crate::utils::convert::*;

pub type Amount = u128;
pub type Epoch = u64;
pub type PrivateKey = ckb_types::H256;

pub type EthAddress = H160;
pub type Staker = H160;
pub type Delegator = H160;
pub type StakerEthAddr = H160;

pub type InStakeSmt = bool;
pub type InDelegateSmt = bool;
pub type NonTopStakers = HashMap<Staker, InStakeSmt>;
pub type NonTopDelegators = HashMap<Delegator, HashMap<Staker, InDelegateSmt>>;

#[derive(Clone)]
pub struct CkbNetwork<C: CkbRpc> {
    pub network_type: NetworkType,
    pub client:       C,
}

#[derive(Clone)]
pub enum NetworkType {
    Mainnet,
    Testnet,
}

#[derive(Default)]
pub struct StakeDelegate {
    pub dividend_ratio:     u8,
    pub maximum_delegators: u32,
    pub threshold:          u128,
}

#[derive(Clone)]
pub struct DelegateItem {
    pub staker:             H160,
    pub is_increase:        bool,
    pub amount:             Amount,
    pub total_amount:       Amount,
    pub inauguration_epoch: Epoch,
}

impl From<&DelegateItem> for DelegateInfoDelta {
    fn from(delegate: &DelegateItem) -> Self {
        DelegateInfoDelta::new_builder()
            .staker(Identity::new_unchecked(
                delegate.staker.as_bytes().to_owned().into(),
            ))
            .is_increase(Byte::new(delegate.is_increase.into()))
            .amount(to_uint128(delegate.amount))
            .inauguration_epoch(to_uint64(delegate.inauguration_epoch))
            .build()
    }
}

#[derive(Clone)]
pub struct StakeItem {
    pub is_increase:        bool,
    pub amount:             Amount,
    pub inauguration_epoch: Epoch,
}

#[derive(Default)]
pub struct Checkpoint {
    pub epoch:               Epoch,
    pub period:              u32,
    pub state_root:          H256,
    pub latest_block_height: u64,
    pub latest_block_hash:   H256,
    pub timestamp:           u64,
    pub proof:               Proof,
    pub propose_count:       Vec<ProposeCount>,
}

impl From<&Checkpoint> for CheckpointCellData {
    fn from(checkpoint: &Checkpoint) -> Self {
        CheckpointCellData::new_builder()
            .version(Byte::default())
            .epoch(to_uint64(checkpoint.epoch))
            .period(to_uint32(checkpoint.period))
            .state_root(Byte32::from_slice(checkpoint.state_root.as_bytes()).unwrap())
            .latest_block_height(to_uint64(checkpoint.latest_block_height))
            .latest_block_hash(Byte32::from_slice(checkpoint.latest_block_hash.as_bytes()).unwrap())
            .timestamp(to_uint64(checkpoint.timestamp))
            .propose_count(propose_counts(&checkpoint.propose_count))
            .build()
    }
}

fn propose_counts(proposes: &[ProposeCount]) -> ProposeCounts {
    let mut propose_count = ProposeCounts::new_builder();
    for propose in proposes.iter() {
        propose_count = propose_count.push(propose.into());
    }
    propose_count.build()
}

#[derive(Default)]
pub struct Proof {
    pub number:     u64,
    pub round:      u64,
    pub block_hash: H256,
    pub signature:  bytes::Bytes,
    pub bitmap:     bytes::Bytes,
}

pub struct ProposeCount {
    pub proposer: H160,
    pub count:    u32,
}

impl From<&ProposeCount> for AProposeCount {
    fn from(propose: &ProposeCount) -> Self {
        AProposeCount::new_builder()
            .address(Byte20::from_slice(propose.proposer.as_bytes()).unwrap())
            .count(to_uint32(propose.count))
            .build()
    }
}

#[derive(Clone, Default)]
pub struct Metadata {
    pub epoch_len:       u32,
    pub period_len:      u32,
    pub quorum:          u16,
    pub gas_limit:       u64,
    pub gas_price:       u64,
    pub interval:        u32,
    pub validators:      Vec<Validator>,
    pub propose_ratio:   u32,
    pub prevote_ratio:   u32,
    pub precommit_ratio: u32,
    pub brake_ratio:     u32,
    pub tx_num_limit:    u32,
    pub max_tx_size:     u32,
    pub block_height:    u64,
}

#[derive(Clone)]
pub struct Validator {
    pub bls_pub_key:    bytes::Bytes,
    pub pub_key:        bytes::Bytes,
    pub address:        H160,
    pub propose_weight: u32,
    pub vote_weight:    u32,
    pub propose_count:  u64,
}

impl From<&Validator> for AValidator {
    fn from(validator: &Validator) -> Self {
        AValidator::new_builder()
            .bls_pub_key(Byte97::from_slice(&validator.bls_pub_key).unwrap())
            .pub_key(Byte65::from_slice(&validator.pub_key).unwrap())
            .address(Identity::from_slice(validator.address.as_bytes()).unwrap())
            .propose_weight(to_uint32(validator.propose_weight))
            .vote_weight(to_uint32(validator.vote_weight))
            .propose_count(to_uint64(validator.propose_count))
            .build()
    }
}

#[derive(Clone, Default)]
pub struct TypeIds {
    pub issue_type_id:        H256,
    pub selection_type_id:    H256,
    pub metadata_type_id:     H256,
    pub checkpoint_type_id:   H256,
    pub stake_smt_type_id:    H256,
    pub delegate_smt_type_id: H256,
    pub reward_type_id:       H256,
    pub xudt_lock_id:         H256,
}

impl From<TypeIds> for MetaTypeIds {
    fn from(type_ids: TypeIds) -> Self {
        MetaTypeIds::new_builder()
            .metadata_type_id(to_byte32(&type_ids.metadata_type_id))
            .checkpoint_type_id(to_byte32(&type_ids.checkpoint_type_id))
            .stake_smt_type_id(to_byte32(&type_ids.stake_smt_type_id))
            .delegate_smt_type_id(to_byte32(&type_ids.delegate_smt_type_id))
            .reward_type_id(to_byte32(&type_ids.reward_type_id))
            .xudt_type_id(to_byte32(&type_ids.xudt_lock_id))
            .build()
    }
}

impl From<&Metadata> for AMetadata {
    fn from(metadata: &Metadata) -> Self {
        AMetadata::new_builder()
            .epoch_len(to_uint32(metadata.epoch_len))
            .period_len(to_uint32(metadata.period_len))
            .quorum(to_uint16(metadata.quorum))
            .gas_limit(to_uint64(metadata.gas_limit))
            .gas_price(to_uint64(metadata.gas_price))
            .interval(to_uint32(metadata.interval))
            .validators(gen_validators(&metadata.validators))
            .propose_ratio(to_uint32(metadata.propose_ratio))
            .prevote_ratio(to_uint32(metadata.prevote_ratio))
            .precommit_ratio(to_uint32(metadata.precommit_ratio))
            .brake_ratio(to_uint32(metadata.brake_ratio))
            .tx_num_limit(to_uint32(metadata.tx_num_limit))
            .max_tx_size(to_uint32(metadata.max_tx_size))
            .block_height(to_uint64(metadata.block_height))
            .build()
    }
}

#[derive(Clone, Default)]
pub struct MetadataCellData {
    pub version:                u8,
    pub epoch:                  u64,
    pub propose_count_smt_root: H256,
    pub type_ids:               TypeIds,
    pub metadata:               Vec<Metadata>,
}

impl From<MetadataCellData> for AMetadataCellData {
    fn from(v: MetadataCellData) -> Self {
        AMetadataCellData::new_builder()
            .version(v.version.into())
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
    pub version:          u8,
    pub smt_roots:        Vec<StakerSmtRoot>, // smt root of all delegator infos
    pub metadata_type_id: H256,
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
    pub smt_root:         H256,
    pub version:          u8,
    pub metadata_type_id: H256,
}

#[derive(Clone, Default)]
pub struct StakeAtCellData {
    pub version:          u8,
    pub l1_address:       H160,
    pub l2_address:       H160,
    pub stake_info:       StakeInfoDelta,
    pub metadata_type_id: H256,
}

#[derive(Clone, Default)]
pub struct StakeInfoDelta {
    pub is_increase:        u8,
    pub amount:             u128,
    pub inauguration_epoch: u64,
}

#[derive(Clone, Default)]
pub struct DelegateAtCellData {
    pub version:          u8,
    pub l1_address:       H160,
    pub metadata_type_id: H256,
    pub delegator_infos:  Vec<DelegateItem>,
}

impl From<DelegateAtCellData> for ADelegateAtCellData {
    fn from(value: DelegateAtCellData) -> Self {
        let infos = DelegateInfoDeltas::new_builder()
            .extend(value.delegator_infos.iter().map(Into::into))
            .build();
        ADelegateAtCellData::new_builder()
            .version(value.version.into())
            .l1_address(Identity::from_slice(value.l1_address.as_bytes()).unwrap())
            .metadata_type_id(to_byte32(&value.metadata_type_id))
            .delegator_infos(infos)
            .build()
    }
}
#[derive(Clone, Default)]
pub struct WithdrawAtCellData {
    pub version:          u8,
    pub metadata_type_id: H256,
    pub withdraw_infos:   Vec<WithdrawInfo>,
}

impl From<WithdrawAtCellData> for AWithdrawAtCellData {
    fn from(value: WithdrawAtCellData) -> Self {
        let infos: AWithdrawInfos = AWithdrawInfos::new_builder()
            .extend(value.withdraw_infos.into_iter().map(Into::into))
            .build();
        AWithdrawAtCellData::new_builder()
            .version(value.version.into())
            .metadata_type_id(to_byte32(&value.metadata_type_id))
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

fn gen_validators(validators: &[Validator]) -> ValidatorList {
    let mut validator_list = ValidatorList::new_builder();
    for validator in validators.iter() {
        validator_list = validator_list.push(validator.into());
    }
    validator_list.build()
}
