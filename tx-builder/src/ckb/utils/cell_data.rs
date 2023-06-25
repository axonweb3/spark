use axon_types::{basic::*, delegate::*, stake::*, withdraw::*};
use bytes::Bytes;
use ckb_types::{
    prelude::{Builder, Entity},
    H160,
};

use common::types::{
    ckb_rpc_client::Cell,
    tx_builder::{DelegateItem, Epoch, StakeItem},
};
use common::utils::convert::*;

use crate::ckb::define::constants::TOKEN_BYTES;
use crate::ckb::define::types::WithdrawInfo as SWithdrawInfo;

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
    let mut res = amount.to_le_bytes().to_vec();
    res.extend(extra_args.to_vec());
    bytes::Bytes::from(res)
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

pub fn update_withdraw_data(
    withdraw_cell: Cell,
    current_epoch: Epoch,
    new_amount: u128,
) -> bytes::Bytes {
    let mut withdraw_data = withdraw_cell.output_data.unwrap().into_bytes();
    let mut total_withdraw_amount = new_u128(&withdraw_data[..TOKEN_BYTES]);
    let cell_withdraws = WithdrawAtCellData::new_unchecked(withdraw_data.split_off(TOKEN_BYTES));

    let mut new_withdraw_infos = WithdrawInfos::new_builder();
    let mut is_inserted = false;

    for item in cell_withdraws.withdraw_infos() {
        let epoch = to_u64(&item.epoch());
        new_withdraw_infos = new_withdraw_infos.push(if epoch == current_epoch {
            is_inserted = true;
            total_withdraw_amount += new_amount;
            WithdrawInfo::from(SWithdrawInfo {
                epoch:  current_epoch,
                amount: to_u128(&item.amount()) + new_amount,
            })
        } else {
            item
        });
    }

    if !is_inserted {
        new_withdraw_infos = new_withdraw_infos.push(WithdrawInfo::from(SWithdrawInfo {
            epoch:  current_epoch,
            amount: new_amount,
        }));
    }

    token_cell_data(total_withdraw_amount, new_withdraw_infos.build().as_bytes())
}
