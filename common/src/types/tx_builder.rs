use std::collections::HashMap;

use axon_types::{
    basic::{Byte20, Byte32, Identity},
    checkpoint::{CheckpointCellData, ProposeCount as AProposeCount, ProposeCounts},
    delegate::DelegateInfoDelta,
};
use ckb_types::H160;
use ethereum_types::H256;
use molecule::prelude::{Builder, Byte, Entity};

use crate::utils::convert::*;

pub type Amount = u128;
pub type Epoch = u64;
pub type PrivateKey = ckb_types::H256;

pub type Address = H160;
pub type Staker = H160;
pub type Delegator = H160;

pub type InStakeSmt = bool;
pub type InDelegateSmt = bool;
pub type NonTopStakers = HashMap<Staker, InStakeSmt>;
pub type NonTopDelegators = HashMap<Delegator, HashMap<Staker, InDelegateSmt>>;

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
