use axon_types::{basic::*, delegate::*, reward::RewardSmtCellData, stake::*, withdraw::*};
use bytes::{BufMut, Bytes};
use ckb_types::{
    bytes::BytesMut,
    prelude::{Builder, Entity},
    H160,
};

use common::types::tx_builder::{
    Amount, DelegateItem, DelegateRequirement as CDelegateRequirement, Epoch, StakeItem,
};
use common::utils::convert::*;

use crate::ckb::define::config::TOKEN_BYTES;
use crate::ckb::define::types::WithdrawInfo as CWithdrawInfo;

pub fn stake_cell_data(
    is_increase: bool,
    amount: Amount,
    inauguration_epoch: Epoch,
) -> StakeAtCellData {
    StakeAtCellData::new_builder()
        .delta(
            (&StakeItem {
                is_increase,
                amount,
                inauguration_epoch,
            })
                .into(),
        )
        .build()
}

pub fn stake_delegate_cell_data(
    threshold: u128,
    maximum_delegators: u32,
    commission_rate: u8,
) -> DelegateCellData {
    DelegateCellData::new_builder()
        .delegate_requirement(
            CDelegateRequirement {
                threshold,
                maximum_delegators,
                commission_rate,
            }
            .into(),
        )
        .build()
}

pub fn delegate_cell_data(delegates: &[DelegateItem]) -> DelegateAtCellData {
    let mut delegator_infos = DelegateInfoDeltas::new_builder();
    for item in delegates.iter() {
        delegator_infos = delegator_infos.push(item.into())
    }

    DelegateAtCellData::new_builder()
        .delegator_infos(delegator_infos.build())
        .build()
}

pub fn delegate_item(delegate: &DelegateInfoDelta) -> DelegateItem {
    DelegateItem {
        staker:             to_h160(&delegate.staker()),
        total_amount:       to_u128(&delegate.total_amount()),
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

pub fn token_cell_data(amount: u128, extra_args: Bytes) -> Bytes {
    let mut data = BytesMut::with_capacity(16 + extra_args.len());
    data.put(&amount.to_le_bytes()[..]);
    data.put(extra_args.as_ref());
    data.freeze()
}

pub fn stake_smt_cell_data(root: Byte32) -> StakeSmtCellData {
    StakeSmtCellData::new_builder().smt_root(root).build()
}

pub fn delegate_smt_cell_data(roots: Vec<(H160, Byte32)>) -> DelegateSmtCellData {
    DelegateSmtCellData::new_builder()
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

pub fn _reward_smt_cell_data(root: Byte32) -> RewardSmtCellData {
    RewardSmtCellData::new_builder()
        .claim_smt_root(root)
        .build()
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
            WithdrawInfo::from(CWithdrawInfo {
                epoch:  current_epoch,
                amount: to_u128(&item.amount()) + new_amount,
            })
        } else {
            item
        });
    }

    token_cell_data(total_withdraw_amount, new_withdraw_infos.build().as_bytes())
}
