use ckb_types::{H160, H256};
use molecule::prelude::{Builder, Entity};

use common::types::axon_types::{
    basic::{Byte32, Byte48, Byte65, Identity},
    delegate::{
        DelegateAtCellData as ADelegateAtCellData,
        DelegateAtCellLockData as ADelegateAtCellLockData, DelegateInfoDeltas,
        DelegateSmtCellData as ADelegateSmtCellData, StakerSmtRoot as AStakerSmtRoot,
        StakerSmtRoots,
    },
    metadata::{MetadataCellData as AMetadataCellData, MetadataList},
    reward::{
        EpochRewardStakeInfo as AEpochRewardStakeInfo,
        EpochRewardStakeInfos as AEpochRewardStakeInfos, NotClaimInfo as ANotClaimInfo,
        RewardDelegateInfo as ARewardDelegateInfo, RewardDelegateInfos as ARewardDelegateInfos,
        RewardSmtCellData as ARewardSmtCellData, RewardStakeInfo as ARewardStakeInfo,
        RewardStakeInfos as ARewardStakeInfos, RewardWitness as ARewardWitness,
    },
    stake::{
        StakeAtCellData as AStakeAtCellData, StakeAtCellLockData as AStakeAtCellLockData,
        StakeInfo as AStakeInfo, StakeInfos, StakeSmtCellData as AStakeSmtCellData,
        StakeSmtUpdateInfo as AStakeSmtUpdateInfo,
    },
    withdraw::{
        WithdrawAtCellData as AWithdrawAtCellData,
        WithdrawAtCellLockData as AWithdrawAtCellLockData, WithdrawInfo as AWithdrawInfo,
        WithdrawInfos as AWithdrawInfos,
    },
};
use common::types::smt::{Proof, Root as SmtRoot};
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
    pub root:   SmtRoot,
}

impl From<StakerSmtRoot> for AStakerSmtRoot {
    fn from(value: StakerSmtRoot) -> Self {
        AStakerSmtRoot::new_builder()
            .staker(to_identity(&value.staker))
            .root(Byte32::from_slice(value.root.as_slice()).unwrap())
            .build()
    }
}

#[derive(Clone, Default)]
pub struct DelegateSmtCellData {
    // pub version:          u8, // useless
    pub metadata_type_id: H256,
    pub smt_roots:        Vec<StakerSmtRoot>, // smt root of all delegator infos
}

impl From<DelegateSmtCellData> for ADelegateSmtCellData {
    fn from(value: DelegateSmtCellData) -> Self {
        ADelegateSmtCellData::new_builder()
            // .version(value.version.into()) // useless
            .metadata_type_id(to_byte32(&value.metadata_type_id))
            .smt_roots({
                let mut list = StakerSmtRoots::new_builder();
                for r in value.smt_roots.into_iter() {
                    list = list.push(r.into());
                }
                list.build()
            })
            .build()
    }
}

#[derive(Clone, Default)]
pub struct StakeInfo {
    pub addr:   H160,
    pub amount: u128,
}

impl From<StakeInfo> for AStakeInfo {
    fn from(value: StakeInfo) -> Self {
        AStakeInfo::new_builder()
            .addr(to_identity(&value.addr))
            .amount(to_uint128(value.amount))
            .build()
    }
}

#[derive(Clone, Default)]
pub struct StakeSmtUpdateInfo {
    pub all_stake_infos: Vec<StakeInfo>,
    pub old_epoch_proof: Vec<u8>,
    pub new_epoch_proof: Vec<u8>,
}

impl From<StakeSmtUpdateInfo> for AStakeSmtUpdateInfo {
    fn from(value: StakeSmtUpdateInfo) -> Self {
        AStakeSmtUpdateInfo::new_builder()
            .all_stake_infos({
                let mut list = StakeInfos::new_builder();
                for info in value.all_stake_infos.into_iter() {
                    list = list.push(AStakeInfo::from(info));
                }
                list.build()
            })
            .old_epoch_proof(to_bytes(value.old_epoch_proof))
            .new_epoch_proof(to_bytes(value.new_epoch_proof))
            .build()
    }
}

#[derive(Clone, Default)]
pub struct StakeSmtCellData {
    // pub version:          u8, // useless
    pub metadata_type_id: H256,
    pub smt_root:         SmtRoot,
}

impl From<StakeSmtCellData> for AStakeSmtCellData {
    fn from(v: StakeSmtCellData) -> Self {
        AStakeSmtCellData::new_builder()
            .smt_root(Byte32::from_slice(v.smt_root.as_slice()).unwrap())
            .metadata_type_id(to_byte32(&v.metadata_type_id))
            .build()
    }
}

#[derive(Clone)]
pub struct StakeAtCellData {
    pub lock: StakeAtCellLockData,
}

impl From<StakeAtCellData> for AStakeAtCellData {
    fn from(value: StakeAtCellData) -> Self {
        AStakeAtCellData::new_builder()
            .lock(value.lock.into())
            .build()
    }
}

#[derive(Clone)]
pub struct StakeAtCellLockData {
    // pub version:          u8, // useless
    // pub l1_address:       H160, // useless
    // pub l2_address:       H160, // useless
    // pub metadata_type_id: H256, // useless
    pub l1_pub_key:  Byte65,
    pub bls_pub_key: Byte48,
    pub stake_info:  StakeItem,
}

impl From<StakeAtCellLockData> for AStakeAtCellLockData {
    fn from(value: StakeAtCellLockData) -> Self {
        AStakeAtCellLockData::new_builder()
            .l1_pub_key(value.l1_pub_key)
            .bls_pub_key(value.bls_pub_key)
            .delta(value.stake_info.into())
            .build()
    }
}

#[derive(Clone, Default)]
pub struct DelegateAtCellData {
    pub lock: DelegateAtCellLockData,
}

impl From<DelegateAtCellData> for ADelegateAtCellData {
    fn from(value: DelegateAtCellData) -> Self {
        ADelegateAtCellData::new_builder()
            .lock(value.lock.into())
            .build()
    }
}

#[derive(Clone, Default)]
pub struct DelegateAtCellLockData {
    // pub version:          u8, // useless
    // pub l1_address:       H160, // useless
    // pub metadata_type_id: H256, // useless
    pub delegator_infos: Vec<DelegateItem>,
}

impl From<DelegateAtCellLockData> for ADelegateAtCellLockData {
    fn from(value: DelegateAtCellLockData) -> Self {
        let infos = DelegateInfoDeltas::new_builder()
            .extend(value.delegator_infos.into_iter().map(Into::into))
            .build();
        ADelegateAtCellLockData::new_builder()
            // .version(value.version.into()) // useless
            // .l1_address(Identity::from_slice(value.l1_address.as_bytes()).unwrap()) // useless
            // .metadata_type_id(to_byte32(&value.metadata_type_id)) // useless
            .delegator_infos(infos)
            .build()
    }
}

#[derive(Clone, Default)]
pub struct WithdrawAtCellData {
    pub lock: WithdrawAtCellLockData,
}

impl From<WithdrawAtCellData> for AWithdrawAtCellData {
    fn from(value: WithdrawAtCellData) -> Self {
        AWithdrawAtCellData::new_builder()
            .lock(value.lock.into())
            .build()
    }
}

#[derive(Clone, Default)]
pub struct WithdrawAtCellLockData {
    // pub version:          u8, // useless
    // pub metadata_type_id: H256, // useless
    pub withdraw_infos: Vec<WithdrawInfo>,
}

impl From<WithdrawAtCellLockData> for AWithdrawAtCellLockData {
    fn from(value: WithdrawAtCellLockData) -> Self {
        let infos: AWithdrawInfos = AWithdrawInfos::new_builder()
            .extend(value.withdraw_infos.into_iter().map(Into::into))
            .build();
        AWithdrawAtCellLockData::new_builder()
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
            .unlock_epoch(to_uint64(value.epoch))
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
                for m in v.metadata.into_iter() {
                    list = list.push(m.into());
                }
                list.build()
            })
            .build()
    }
}

#[derive(Default)]
pub struct RewardSmtCellData {
    pub claim_smt_root:   SmtRoot,
    pub metadata_type_id: H256,
}

impl From<RewardSmtCellData> for ARewardSmtCellData {
    fn from(v: RewardSmtCellData) -> Self {
        ARewardSmtCellData::new_builder()
            .claim_smt_root(Byte32::from_slice(v.claim_smt_root.as_slice()).unwrap())
            .metadata_type_id(to_byte32(&v.metadata_type_id))
            .build()
    }
}

#[derive(Default)]
pub struct RewardWitness {
    pub miner:              H160,
    pub old_not_claim_info: NotClaimInfo,
    pub reward_infos:       Vec<EpochRewardStakeInfo>,
    pub new_not_claim_info: NotClaimInfo,
}

impl From<RewardWitness> for ARewardWitness {
    fn from(v: RewardWitness) -> Self {
        ARewardWitness::new_builder()
            .miner(to_identity(&v.miner))
            .old_not_claim_info(v.old_not_claim_info.into())
            .new_not_claim_info(v.new_not_claim_info.into())
            .reward_infos({
                let mut list = AEpochRewardStakeInfos::new_builder();
                for r in v.reward_infos.into_iter() {
                    list = list.push(r.into());
                }
                list.build()
            })
            .build()
    }
}

#[derive(Default)]
pub struct NotClaimInfo {
    pub epoch: u64,
    pub proof: Proof,
}

impl From<NotClaimInfo> for ANotClaimInfo {
    fn from(v: NotClaimInfo) -> Self {
        ANotClaimInfo::new_builder()
            .epoch(to_uint64(v.epoch))
            .proof(to_bytes(v.proof))
            .build()
    }
}

#[derive(Default)]
pub struct EpochRewardStakeInfo {
    pub count_proof:        Proof,
    pub count_root:         SmtRoot,
    pub count_epoch_proof:  Proof,
    pub amount_proof:       Proof,
    pub amount_root:        SmtRoot,
    pub amount_epoch_proof: Proof,
    pub reward_stake_infos: Vec<RewardStakeInfo>,
}

impl From<EpochRewardStakeInfo> for AEpochRewardStakeInfo {
    fn from(v: EpochRewardStakeInfo) -> Self {
        AEpochRewardStakeInfo::new_builder()
            .count_proof(to_bytes(v.count_proof))
            .count_root(to_bytes(v.count_root.as_slice().to_owned()))
            .count_epoch_proof(to_bytes(v.count_epoch_proof))
            .amount_proof(to_bytes(v.amount_proof))
            .amount_root(to_bytes(v.amount_root.as_slice().to_owned()))
            .amount_epoch_proof(to_bytes(v.amount_epoch_proof))
            .reward_stake_infos({
                let mut list = ARewardStakeInfos::new_builder();
                for r in v.reward_stake_infos.into_iter() {
                    list = list.push(r.into());
                }
                list.build()
            })
            .build()
    }
}

#[derive(Default)]
pub struct RewardStakeInfo {
    pub validator:            H160,
    pub propose_count:        u64,
    pub stake_amount:         u128,
    pub delegate_infos:       Vec<RewardDelegateInfo>,
    pub delegate_epoch_proof: Proof,
}

impl From<RewardStakeInfo> for ARewardStakeInfo {
    fn from(v: RewardStakeInfo) -> Self {
        ARewardStakeInfo::new_builder()
            .validator(to_identity(&v.validator))
            .propose_count(to_uint64(v.propose_count))
            .staker_amount(to_uint128(v.stake_amount))
            .delegate_epoch_proof(to_bytes(v.delegate_epoch_proof))
            .delegate_infos({
                let mut list = ARewardDelegateInfos::new_builder();
                for r in v.delegate_infos.into_iter() {
                    list = list.push(r.into());
                }
                list.build()
            })
            .build()
    }
}

#[derive(Default)]
pub struct RewardDelegateInfo {
    pub delegator_addr: ethereum_types::H160,
    pub amount:         u128,
}

impl From<RewardDelegateInfo> for ARewardDelegateInfo {
    fn from(v: RewardDelegateInfo) -> Self {
        ARewardDelegateInfo::new_builder()
            .delegator_addr(Identity::new_unchecked(bytes::Bytes::from(
                v.delegator_addr.as_bytes().to_owned(),
            )))
            .amount(to_uint128(v.amount))
            .build()
    }
}
