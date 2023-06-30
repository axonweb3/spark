use anyhow::Result;
use ckb_types::packed::{CellDep, OutPoint, Script};
use ckb_types::prelude::{Builder, Entity, Pack};
use ckb_types::{H160, H256};

use common::traits::ckb_rpc_client::CkbRpc;
use common::types::axon_types::withdraw::WithdrawArgs;
use common::types::ckb_rpc_client::Cell;
use common::types::tx_builder::NetworkType;
use common::utils::convert::*;

use crate::ckb::define::scripts::*;
use crate::ckb::helper::ckb::cell_collector::get_cell_by_scripts;
use crate::ckb::helper::metadata::Metadata;
use crate::ckb::NETWORK_TYPE;
use crate::{cell_dep, out_point, script};

use common::types::{
    axon_types::withdraw::{
        WithdrawAtCellData as AWithdrawAtCellData, WithdrawInfo as AWithdrawInfo,
        WithdrawInfos as AWithdrawInfos,
    },
    tx_builder::Epoch,
};

use crate::ckb::define::constants::TOKEN_BYTES;
use crate::ckb::define::types::WithdrawInfo;
use crate::ckb::helper::token_cell_data;

pub struct Withdraw;

impl Withdraw {
    pub fn lock(metadata_type_id: &H256, addr: &H160) -> Script {
        let metadata_type_hash = Metadata::type_(metadata_type_id).calc_script_hash();
        let args = WithdrawArgs::new_builder()
            .metadata_type_id(to_axon_byte32(&metadata_type_hash))
            .addr(to_identity(addr))
            .build()
            .as_bytes();

        match **NETWORK_TYPE.load() {
            NetworkType::Mainnet => script!(
                &WITHDRAW_LOCK_MAINNET.code_hash,
                WITHDRAW_LOCK_MAINNET.hash_type,
                args
            ),
            NetworkType::Testnet => script!(
                &WITHDRAW_LOCK_TESTNET.code_hash,
                WITHDRAW_LOCK_TESTNET.hash_type,
                args
            ),
        }
    }

    pub fn lock_dep() -> CellDep {
        match **NETWORK_TYPE.load() {
            NetworkType::Mainnet => cell_dep!(
                &WITHDRAW_LOCK_MAINNET.tx_hash,
                WITHDRAW_LOCK_MAINNET.index,
                WITHDRAW_LOCK_MAINNET.dep_type
            ),
            NetworkType::Testnet => cell_dep!(
                &WITHDRAW_LOCK_TESTNET.tx_hash,
                WITHDRAW_LOCK_TESTNET.index,
                WITHDRAW_LOCK_TESTNET.dep_type
            ),
        }
    }

    pub async fn get_cell(
        ckb_rpc: &impl CkbRpc,
        withdraw_lock: Script,
        xudt: Script,
    ) -> Result<Option<Cell>> {
        get_cell_by_scripts(ckb_rpc, withdraw_lock, xudt).await
    }

    pub fn update_cell_data(
        withdraw_cell: Cell,
        inaugration_epoch: Epoch,
        new_amount: u128,
    ) -> bytes::Bytes {
        let mut withdraw_data = withdraw_cell.output_data.unwrap().into_bytes();
        let mut total_withdraw_amount = new_u128(&withdraw_data[..TOKEN_BYTES]);
        let withdraw_data =
            AWithdrawAtCellData::new_unchecked(withdraw_data.split_off(TOKEN_BYTES));

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
}
