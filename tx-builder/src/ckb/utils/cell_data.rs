use axon_types::delegate::DelegateInfoDelta;
use axon_types::stake::StakeInfoDelta;
use axon_types::withdraw::{
    WithdrawAtCellData as AWithdrawAtCellData, WithdrawInfo as AWithdrawInfo,
    WithdrawInfos as AWithdrawInfos,
};
use bytes::Bytes;
use ckb_types::prelude::{Builder, Entity};

use common::types::{
    ckb_rpc_client::Cell,
    tx_builder::{DelegateItem, Epoch, StakeItem},
};
use common::utils::convert::*;

use crate::ckb::define::constants::TOKEN_BYTES;
use crate::ckb::define::types::WithdrawInfo;

pub fn stake_item(stake: &StakeInfoDelta) -> StakeItem {
    StakeItem {
        is_increase:        to_bool(&stake.is_increase()),
        amount:             to_u128(&stake.amount()),
        inauguration_epoch: to_u64(&stake.inauguration_epoch()),
    }
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

pub fn token_cell_data(amount: u128, extra_args: Bytes) -> Bytes {
    let mut res = amount.to_le_bytes().to_vec();
    res.extend(extra_args.to_vec());
    bytes::Bytes::from(res)
}

pub fn update_withdraw_data(
    withdraw_cell: Cell,
    inaugration_epoch: Epoch,
    new_amount: u128,
) -> bytes::Bytes {
    let mut withdraw_data = withdraw_cell.output_data.unwrap().into_bytes();
    let mut total_withdraw_amount = new_u128(&withdraw_data[..TOKEN_BYTES]);
    let withdraw_data = AWithdrawAtCellData::new_unchecked(withdraw_data.split_off(TOKEN_BYTES));

    let mut new_withdraw_infos = AWithdrawInfos::new_builder();
    let mut inserted = false;

    for item in withdraw_data.lock().withdraw_infos() {
        let epoch = to_u64(&item.unlock_epoch());
        new_withdraw_infos = new_withdraw_infos.push(if epoch == inaugration_epoch {
            inserted = true;
            total_withdraw_amount += new_amount;
            AWithdrawInfo::from(WithdrawInfo {
                epoch:  inaugration_epoch,
                amount: to_u128(&item.amount()) + new_amount,
            })
        } else {
            item
        });
    }

    if !inserted {
        new_withdraw_infos = new_withdraw_infos.push(AWithdrawInfo::from(WithdrawInfo {
            epoch:  inaugration_epoch,
            amount: new_amount,
        }));
    }

    let inner_withdraw_data = withdraw_data.lock();

    token_cell_data(
        total_withdraw_amount,
        withdraw_data
            .as_builder()
            .lock(
                inner_withdraw_data
                    .as_builder()
                    .withdraw_infos(new_withdraw_infos.build())
                    .build(),
            )
            .build()
            .as_bytes(),
    )
}
