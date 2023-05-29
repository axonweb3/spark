use std::collections::HashMap;

use axon_types::{
    basic::{Byte20, Byte32, Byte65, Byte97, Identity},
    checkpoint::{CheckpointCellData, ProposeCount as AProposeCount, ProposeCounts},
    delegate::{DelegateInfoDelta, DelegateRequirement as ADelegateRequirement},
    metadata::{
        Metadata as AMetadata, TypeIds as ATypeIds, Validator as AValidator, ValidatorList,
    },
    stake::StakeInfoDelta,
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

#[derive(Clone, Eq, PartialEq)]
pub enum NetworkType {
    Mainnet,
    Testnet,
}

#[derive(Default)]
pub struct DelegateRequirement {
    pub commission_rate:    u8,
    pub maximum_delegators: u32,
    pub threshold:          u128,
}

impl From<DelegateRequirement> for ADelegateRequirement {
    fn from(v: DelegateRequirement) -> Self {
        ADelegateRequirement::new_builder()
            .threshold(to_uint128(v.threshold))
            .max_delegator_size(to_uint32(v.maximum_delegators))
            .commission_rate(v.commission_rate.into())
            .build()
    }
}

#[derive(Clone, Default)]
pub struct StakeItem {
    pub is_increase:        bool,
    pub amount:             Amount,
    pub inauguration_epoch: Epoch,
}

impl From<&StakeItem> for StakeInfoDelta {
    fn from(stake: &StakeItem) -> Self {
        StakeInfoDelta::new_builder()
            .is_increase(Byte::new(stake.is_increase.into()))
            .amount(to_uint128(stake.amount))
            .inauguration_epoch(to_uint64(stake.inauguration_epoch))
            .build()
    }
}

#[derive(Clone, Default)]
pub struct DelegateItem {
    pub staker:             H160,
    pub total_amount:       Amount, // delegate tx does not need to fill this field
    pub is_increase:        bool,
    pub amount:             Amount,
    pub inauguration_epoch: Epoch,
}

impl DelegateItem {
    pub fn new_for_delegate(
        staker: H160,
        is_increase: bool,
        amount: Amount,
        inauguration_epoch: Epoch,
    ) -> Self {
        Self {
            staker,
            is_increase,
            amount,
            inauguration_epoch,
            total_amount: 0,
        }
    }
}

impl From<&DelegateItem> for DelegateInfoDelta {
    fn from(delegate: &DelegateItem) -> Self {
        DelegateInfoDelta::new_builder()
            .staker(Identity::new_unchecked(
                delegate.staker.as_bytes().to_owned().into(),
            ))
            .total_amount(to_uint128(delegate.total_amount))
            .is_increase(Byte::new(delegate.is_increase.into()))
            .amount(to_uint128(delegate.amount))
            .inauguration_epoch(to_uint64(delegate.inauguration_epoch))
            .build()
    }
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

#[derive(Clone, Default, Debug)]
pub struct StakeTypeIds {
    pub metadata_type_id:   H256,
    pub checkpoint_type_id: H256,
    pub xudt_owner:         H256,
}

#[derive(Clone, Default)]
pub struct TypeIds {
    pub issue_type_id:        H256,
    pub selection_type_id:    H256,
    pub metadata_code_hash:   H256,
    pub metadata_type_id:     H256,
    pub checkpoint_code_hash: H256,
    pub checkpoint_type_id:   H256,
    pub stake_code_hash:      H256,
    pub stake_smt_type_id:    H256,
    pub delegate_code_hash:   H256,
    pub delegate_smt_type_id: H256,
    pub reward_code_hash:     H256,
    pub reward_smt_type_id:   H256,
    pub withdraw_code_hash:   H256,
    pub xudt_type_hash:       H256,
    pub xudt_owner:           H256,
}

impl From<TypeIds> for ATypeIds {
    fn from(type_ids: TypeIds) -> Self {
        ATypeIds::new_builder()
            .issue_type_id(to_byte32(&type_ids.issue_type_id))
            .selection_type_id(to_byte32(&type_ids.selection_type_id))
            .metadata_code_hash(to_byte32(&type_ids.metadata_code_hash))
            .metadata_type_id(to_byte32(&type_ids.metadata_type_id))
            .checkpoint_code_hash(to_byte32(&type_ids.checkpoint_code_hash))
            .checkpoint_type_id(to_byte32(&type_ids.checkpoint_type_id))
            .stake_smt_code_hash(to_byte32(&type_ids.stake_code_hash))
            .stake_smt_type_id(to_byte32(&type_ids.stake_smt_type_id))
            .delegate_smt_code_hash(to_byte32(&type_ids.delegate_code_hash))
            .delegate_smt_type_id(to_byte32(&type_ids.delegate_smt_type_id))
            .reward_code_hash(to_byte32(&type_ids.reward_code_hash))
            .reward_type_id(to_byte32(&type_ids.reward_smt_type_id))
            .withdraw_code_hash(to_byte32(&type_ids.withdraw_code_hash))
            .xudt_type_hash(to_byte32(&type_ids.xudt_type_hash))
            .xudt_owner_lock_hash(to_byte32(&type_ids.xudt_owner))
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

fn gen_validators(validators: &[Validator]) -> ValidatorList {
    let mut validator_list = ValidatorList::new_builder();
    for validator in validators.iter() {
        validator_list = validator_list.push(validator.into());
    }
    validator_list.build()
}
