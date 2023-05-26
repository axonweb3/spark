use axon_types::{
    basic::*, delegate::*, metadata::*, reward::RewardSmtCellData, stake::*, withdraw::*,
};
use ckb_types::{
    prelude::{Builder, Entity, Pack},
    H160,
};
use molecule::prelude::Byte;

use common::types::tx_builder::{
    Amount, DelegateItem, Epoch, Metadata as TMetadata, StakeItem, TypeIds,
};
use common::utils::convert::*;

use crate::ckb::define::config::TOKEN_BYTES;

pub fn stake_token_cell_data(
    is_increase: bool,
    amount: Amount,
    inauguration_epoch: Epoch,
) -> StakeAtCellData {
    StakeAtCellData::new_builder()
        .version(Byte::default())
        .l1_address(Identity::default())  // todo
        .l2_address(Identity::default())  // todo
        .metadata_type_id(Byte32::default()) // todo
        .stake_info(StakeInfoDelta::new_builder()
            .is_increase(Byte::new(is_increase.into()))
            .amount(to_uint128(amount))
            .inauguration_epoch(to_uint64(inauguration_epoch))
            .build()
        )
        .build()
}

pub fn delegate_cell_data(
    threshold: u128,
    max_delegator_size: u32,
    dividend_ratio: u8,
) -> DelegateCellData {
    DelegateCellData::new_builder()
        .version(Byte::default())
        .l1_address(Identity::default())  // todo
        .l2_address(Identity::default())  // todo
        .delegate_requirement(DelegateRequirement::new_builder()
            .threshold(to_uint128(threshold))
            .max_delegator_size(to_uint32(max_delegator_size))
            .dividend_ratio(dividend_ratio.into())
            .build()
        )
        .metadata_type_id(Byte32::default())  // todo
        .build()
}

pub fn delegate_token_cell_data(delegates: &[DelegateItem]) -> DelegateAtCellData {
    let mut delegator_infos = DelegateInfoDeltas::new_builder();
    for item in delegates.iter() {
        delegator_infos = delegator_infos.push(item.into())
    }

    DelegateAtCellData::new_builder()
        .version(Byte::default())
        .l1_address(Identity::default())  // todo
        .metadata_type_id(Byte32::default())  // todo
        .delegator_infos(delegator_infos.build())
        .build()
}

pub fn withdraw_token_cell_data(withdraw_infos: Option<WithdrawInfos>) -> WithdrawAtCellData {
    WithdrawAtCellData::new_builder()
        .version(Byte::default())
        .metadata_type_id(Byte32::default())  // todo
        .withdraw_infos(withdraw_infos.unwrap_or_default())
        .build()
}

pub fn withdraw_info(epoch: Epoch, amount: Amount) -> WithdrawInfo {
    WithdrawInfo::new_builder()
        .epoch(to_uint64(epoch))
        .amount(to_uint128(amount))
        .build()
}

pub fn delegate_item(delegate: &DelegateInfoDelta) -> DelegateItem {
    DelegateItem {
        staker:             to_h160(&delegate.staker()),
        is_increase:        to_bool(&delegate.is_increase()),
        amount:             to_u128(&delegate.amount()),
        inauguration_epoch: to_u64(&delegate.inauguration_epoch()),
    }
}

pub fn stake_item(stake: &StakeInfoDelta) -> StakeItem {
    StakeItem {
        is_increase:        to_bool(&stake.is_increase()),
        amount:             to_u128(&stake.amount()),
        inauguration_epoch: to_u64(&stake.inauguration_epoch()),
    }
}

pub fn token_cell_data(amount: u128, other: molecule::bytes::Bytes) -> molecule::bytes::Bytes {
    let total_stake_amount = amount.pack();
    let cell_data = total_stake_amount.as_bytes();
    cell_data.to_vec().extend(other);
    cell_data
}

pub fn stake_smt_cell_data(root: Byte32) -> StakeSmtCellData {
    StakeSmtCellData::new_builder()
        .version(Byte::default())
        .metadata_type_id(Byte32::default())  // todo
        .smt_root(root)
        .build()
}

pub fn delegate_smt_cell_data(roots: Vec<(H160, Byte32)>) -> DelegateSmtCellData {
    DelegateSmtCellData::new_builder()
        .version(Byte::default())
        .metadata_type_id(Byte32::default())  // todo
        .smt_roots(delegate_smt_roots(roots))
        .build()
}

fn delegate_smt_roots(roots: Vec<(H160, Byte32)>) -> StakerSmtRoots {
    let mut smt_roots = StakerSmtRoots::new_builder();
    for (staker, root) in roots.into_iter() {
        smt_roots = smt_roots.push(
            StakerSmtRoot::new_builder()
                .staker(Identity::new_unchecked(staker.as_bytes().to_owned().into()))
                .root(root)
                .build(),
        )
    }
    smt_roots.build()
}

pub fn update_withdraw_data(
    withdraw_data: bytes::Bytes,
    current_epoch: Epoch,
    new_amount: u128,
) -> bytes::Bytes {
    let mut withdraw_data = withdraw_data;
    let mut total_withdraw_amount = new_u128(&withdraw_data[..TOKEN_BYTES]);
    let cell_withdraws = WithdrawAtCellData::new_unchecked(withdraw_data.split_off(TOKEN_BYTES));

    let mut new_withdraw_infos = WithdrawInfos::new_builder();

    for item in cell_withdraws.withdraw_infos() {
        let epoch = to_u64(&item.epoch());
        new_withdraw_infos = new_withdraw_infos.push(if epoch == current_epoch {
            total_withdraw_amount += new_amount;
            withdraw_info(current_epoch, to_u128(&item.amount()) + new_amount)
        } else {
            item
        });
    }

    token_cell_data(total_withdraw_amount, new_withdraw_infos.build().as_bytes())
}

pub fn _reward_smt_cell_data(root: Byte32) -> RewardSmtCellData {
    RewardSmtCellData::new_builder()
        .version(Byte::default())
        .metadata_type_id(Byte32::default()) // todo
        .claim_smt_root(root)
        .build()
}

pub fn metadata_cell_data(
    epoch: Epoch,
    type_ids: TypeIds,
    metadata: &[TMetadata],
    proposal_smt_root: Byte32,
) -> MetadataCellData {
    MetadataCellData::new_builder()
        .version(Byte::default())
        .epoch(to_uint64(epoch))
        .type_ids(type_ids.into())
        .metadata(_gen_metadatas(metadata))
        .propose_count_smt_root(proposal_smt_root)
        .build()
}

fn _gen_metadatas(metadatas: &[TMetadata]) -> MetadataList {
    let mut metadata_list = MetadataList::new_builder();
    for metadata in metadatas.iter() {
        metadata_list = metadata_list.push(metadata.into());
    }
    metadata_list.build()
}
