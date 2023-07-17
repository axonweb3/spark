use std::collections::HashMap;
use std::fmt::{self, Formatter};
use std::str::FromStr;

use axon_types::{
    basic::{Byte20, Byte32, Byte48, Byte65, Identity},
    checkpoint::{CheckpointCellData, ProposeCount as AProposeCount, ProposeCounts},
    delegate::{DelegateInfoDelta, DelegateRequirement as ADelegateRequirement},
    metadata::{
        Metadata as AMetadata, TypeIds as ATypeIds, Validator as AValidator, ValidatorList,
    },
    stake::StakeInfoDelta,
};
use ckb_types::{H160, H256};
use molecule::prelude::{Builder, Byte, Entity, Reader};
use rlp::Encodable;
use rlp_derive::{RlpDecodable, RlpEncodable};
use serde::de::{self, Deserialize, Deserializer, Visitor};

use crate::types::primitive::Hasher;
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NetworkType {
    Mainnet,
    Testnet,
    Devnet,
}

impl<'a> Deserialize<'a> for NetworkType {
    fn deserialize<D>(deserializer: D) -> Result<NetworkType, D::Error>
    where
        D: Deserializer<'a>,
    {
        deserializer.deserialize_any(NetworkTypeVisitor)
    }
}

struct NetworkTypeVisitor;

impl<'a> Visitor<'a> for NetworkTypeVisitor {
    type Value = NetworkType;

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match v {
            "mainnet" | "Mainnet" => Ok(NetworkType::Mainnet),
            "testnet" | "Testnet" => Ok(NetworkType::Testnet),
            _ => Err(de::Error::custom(format!("invalid network type: {}", v))),
        }
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_str(v.as_str())
    }

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("mainnet or testnet")
    }
}

impl FromStr for NetworkType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mainnet" => Ok(NetworkType::Mainnet),
            "testnet" => Ok(NetworkType::Testnet),
            _ => Err(format!("invalid network type: {}", s)),
        }
    }
}

pub struct FirstStakeInfo {
    pub l1_pub_key:  Byte65,
    pub bls_pub_key: Byte48,
    pub delegate:    DelegateRequirement,
}

#[derive(Clone, Default)]
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

#[derive(Clone, Default, Debug)]
pub struct StakeItem {
    pub is_increase:        bool,
    pub amount:             Amount,
    pub inauguration_epoch: Epoch,
}

impl From<StakeItem> for StakeInfoDelta {
    fn from(stake: StakeItem) -> Self {
        StakeInfoDelta::new_builder()
            .is_increase((stake.is_increase as u8).into())
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

impl From<DelegateItem> for DelegateInfoDelta {
    fn from(delegate: DelegateItem) -> Self {
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

#[derive(Default, Clone)]
pub struct Checkpoint {
    pub epoch:               Epoch,
    pub period:              u32,
    pub state_root:          H256,
    pub latest_block_height: u64,
    pub latest_block_hash:   H256,
    pub timestamp:           u64,
    pub propose_count:       Vec<ProposeCount>,
}

impl From<Checkpoint> for CheckpointCellData {
    fn from(checkpoint: Checkpoint) -> Self {
        CheckpointCellData::new_builder()
            .version(Byte::default())
            .epoch(to_uint64(checkpoint.epoch))
            .period(to_uint32(checkpoint.period))
            .state_root(Byte32::from_slice(checkpoint.state_root.as_bytes()).unwrap())
            .latest_block_height(to_uint64(checkpoint.latest_block_height))
            .latest_block_hash(Byte32::from_slice(checkpoint.latest_block_hash.as_bytes()).unwrap())
            .timestamp(to_uint64(checkpoint.timestamp))
            .propose_count({
                let mut list = ProposeCounts::new_builder();
                for p in checkpoint.propose_count.into_iter() {
                    list = list.push(p.into());
                }
                list.build()
            })
            .build()
    }
}

#[derive(Clone, Debug)]
pub struct ProposeCount {
    pub proposer: H160,
    pub count:    u64,
}

impl From<ProposeCount> for AProposeCount {
    fn from(propose: ProposeCount) -> Self {
        AProposeCount::new_builder()
            .address(Byte20::from_slice(propose.proposer.as_bytes()).unwrap())
            .count(to_uint64(propose.count))
            .build()
    }
}

#[derive(Default)]
pub struct CheckpointProof {
    pub proof:    Proof,
    pub proposal: Proposal,
}

#[derive(RlpEncodable, RlpDecodable, Default, Clone)]
pub struct Proof {
    pub number:     u64,
    pub round:      u64,
    pub block_hash: ethereum_types::H256,
    pub signature:  bytes::Bytes,
    pub bitmap:     bytes::Bytes,
}

impl Proof {
    pub fn bytes(&self) -> bytes::Bytes {
        self.rlp_bytes().into()
    }
}

type Hash = ethereum_types::H256;

#[derive(RlpEncodable, RlpDecodable, Default)]
pub struct Proposal {
    pub prev_hash:                ethereum_types::H256,
    pub proposer:                 ethereum_types::H160,
    pub prev_state_root:          ethereum_types::H256,
    pub transactions_root:        ethereum_types::H256,
    pub signed_txs_hash:          ethereum_types::H256,
    pub timestamp:                u64,
    pub number:                   u64,
    pub proof:                    Proof,
    pub call_system_script_count: u32,
    pub tx_hashes:                Vec<Hash>,
}

impl Proposal {
    pub fn bytes(&self) -> bytes::Bytes {
        self.rlp_bytes().into()
    }

    pub fn hash(&self) -> H256 {
        Hasher::digest(self.rlp_bytes().freeze())
    }
}

#[derive(Clone, Default)]
pub struct Metadata {
    pub epoch_len:       u32, // how many periods as one epoch
    pub period_len:      u32, // how many blocks as one period
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

impl From<AMetadata> for Metadata {
    fn from(meta: AMetadata) -> Self {
        let r = meta.as_reader();
        Metadata {
            epoch_len:       to_u32(&r.epoch_len().to_entity()),
            period_len:      to_u32(&r.period_len().to_entity()),
            quorum:          to_u16(&r.quorum().to_entity()),
            gas_limit:       to_u64(&r.gas_limit().to_entity()),
            gas_price:       to_u64(&r.gas_price().to_entity()),
            interval:        to_u32(&r.interval().to_entity()),
            validators:      {
                let mut res = Vec::with_capacity(r.validators().item_count());
                for i in r.validators().to_entity() {
                    res.push(Validator {
                        bls_pub_key:    i.bls_pub_key().as_bytes(),
                        pub_key:        i.pub_key().as_bytes(),
                        address:        H160::from_slice(&i.address().raw_data()).unwrap(),
                        propose_weight: to_u32(&i.propose_weight()),
                        vote_weight:    to_u32(&i.vote_weight()),
                        propose_count:  to_u64(&i.propose_count()),
                    })
                }
                res
            },
            propose_ratio:   to_u32(&r.propose_ratio().to_entity()),
            prevote_ratio:   to_u32(&r.prevote_ratio().to_entity()),
            precommit_ratio: to_u32(&r.precommit_ratio().to_entity()),
            brake_ratio:     to_u32(&r.brake_ratio().to_entity()),
            tx_num_limit:    to_u32(&r.tx_num_limit().to_entity()),
            max_tx_size:     to_u32(&r.max_tx_size().to_entity()),
            block_height:    to_u64(&r.block_height().to_entity()),
        }
    }
}

#[derive(Clone, Default, PartialEq, Eq)]
pub struct Validator {
    pub bls_pub_key:    bytes::Bytes, // Byte48,
    pub pub_key:        bytes::Bytes, // Byte65,
    pub address:        H160,
    pub propose_weight: u32,
    pub vote_weight:    u32,
    pub propose_count:  u64,
}

impl core::cmp::PartialOrd for Validator {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl core::cmp::Ord for Validator {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.address.cmp(&other.address)
    }
}

impl From<Validator> for AValidator {
    fn from(validator: Validator) -> Self {
        AValidator::new_builder()
            .pub_key(Byte65::new_unchecked(validator.pub_key))
            .bls_pub_key(Byte48::new_unchecked(validator.bls_pub_key))
            .address(Identity::from_slice(validator.address.as_bytes()).unwrap())
            .propose_weight(to_uint32(validator.propose_weight))
            .vote_weight(to_uint32(validator.vote_weight))
            .propose_count(to_uint64(validator.propose_count))
            .build()
    }
}

#[derive(Clone, Default, Debug)]
pub struct CheckpointTypeIds {
    pub metadata_type_id:   H256,
    pub checkpoint_type_id: H256,
}

#[derive(Clone, Default, Debug)]
pub struct StakeTypeIds {
    pub metadata_type_id:   H256,
    pub checkpoint_type_id: H256,
    pub xudt_owner:         H256,
}

#[derive(Clone, Default, Debug)]
pub struct RewardTypeIds {
    pub selection_type_id:    H256,
    pub metadata_type_id:     H256,
    pub checkpoint_type_id:   H256,
    pub reward_smt_type_id:   H256,
    pub stake_smt_type_id:    H256,
    pub delegate_smt_type_id: H256,
    pub xudt_owner:           H256,
}

pub struct StakeSmtTypeIds {
    pub metadata_type_id:   H256,
    pub stake_smt_type_id:  H256,
    pub checkpoint_type_id: H256,
    pub xudt_owner:         H256,
}

#[derive(Clone, Default, Debug)]
pub struct DelegateSmtTypeIds {
    pub metadata_type_id:     H256,
    pub delegate_smt_type_id: H256,
    pub checkpoint_type_id:   H256,
    pub xudt_owner:           H256,
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

impl From<Metadata> for AMetadata {
    fn from(metadata: Metadata) -> Self {
        AMetadata::new_builder()
            .epoch_len(to_uint32(metadata.epoch_len))
            .period_len(to_uint32(metadata.period_len))
            .quorum(to_uint16(metadata.quorum))
            .gas_limit(to_uint64(metadata.gas_limit))
            .gas_price(to_uint64(metadata.gas_price))
            .interval(to_uint32(metadata.interval))
            .validators({
                let mut list = ValidatorList::new_builder();
                for v in metadata.validators.into_iter() {
                    list = list.push(v.into());
                }
                list.build()
            })
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

pub struct RewardInfo {
    pub base_reward:               Amount,
    pub half_reward_cycle:         Epoch,
    pub theoretical_propose_count: u64,
    pub epoch_count:               u64,
}
