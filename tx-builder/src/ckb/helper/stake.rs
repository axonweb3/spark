use anyhow::Result;
use bytes::Bytes;
use ckb_types::packed::{CellDep, OutPoint, Script, WitnessArgs};
use ckb_types::prelude::{Builder, Entity, Pack};
use ckb_types::{H160, H256};

use common::traits::ckb_rpc_client::CkbRpc;
use common::types::axon_types::stake::{
    StakeArgs, StakeAtWitness, StakeInfoDelta, StakeSmtWitness as AStakeSmtWitness,
};
use common::types::ckb_rpc_client::Cell;
use common::types::tx_builder::{NetworkType, StakeItem};
use common::utils::convert::*;

use crate::ckb::define::scripts::*;
use crate::ckb::define::types::{StakeInfo, StakeSmtUpdateInfo, StakeSmtWitness};
use crate::ckb::helper::ckb::cell_collector::{get_cell_by_scripts, get_cell_by_type};
use crate::ckb::helper::metadata::Metadata;
use crate::ckb::helper::unique_cell_dep;
use crate::ckb::NETWORK_TYPE;
use crate::{cell_dep, out_point, script};

pub struct Stake;

impl Stake {
    pub fn lock(metadata_type_id: &H256, staker_addr: &H160) -> Script {
        let metadata_type_hash = Metadata::type_(metadata_type_id).calc_script_hash();
        let args = StakeArgs::new_builder()
            .metadata_type_id(to_axon_byte32(&metadata_type_hash))
            .stake_addr(to_identity(staker_addr))
            .build()
            .as_bytes();

        match **NETWORK_TYPE.load() {
            NetworkType::Mainnet => script!(
                &STAKE_LOCK_MAINNET.code_hash,
                STAKE_LOCK_MAINNET.hash_type,
                args
            ),
            NetworkType::Testnet => script!(
                &STAKE_LOCK_TESTNET.code_hash,
                STAKE_LOCK_TESTNET.hash_type,
                args
            ),
        }
    }

    pub fn smt_type(stake_smt_type_id: &H256) -> Script {
        let args = Bytes::from(stake_smt_type_id.as_bytes().to_vec());
        match **NETWORK_TYPE.load() {
            NetworkType::Mainnet => script!(
                &STAKE_SMT_TYPE_MAINNET.code_hash,
                STAKE_SMT_TYPE_MAINNET.hash_type,
                args
            ),
            NetworkType::Testnet => script!(
                &STAKE_SMT_TYPE_TESTNET.code_hash,
                STAKE_SMT_TYPE_TESTNET.hash_type,
                args
            ),
        }
    }

    pub fn lock_dep() -> CellDep {
        match **NETWORK_TYPE.load() {
            NetworkType::Mainnet => cell_dep!(
                &STAKE_LOCK_MAINNET.tx_hash,
                STAKE_LOCK_MAINNET.index,
                STAKE_LOCK_MAINNET.dep_type
            ),
            NetworkType::Testnet => cell_dep!(
                &STAKE_LOCK_TESTNET.tx_hash,
                STAKE_LOCK_TESTNET.index,
                STAKE_LOCK_TESTNET.dep_type
            ),
        }
    }

    pub fn smt_type_dep() -> CellDep {
        match **NETWORK_TYPE.load() {
            NetworkType::Mainnet => cell_dep!(
                &STAKE_SMT_TYPE_MAINNET.tx_hash,
                STAKE_SMT_TYPE_MAINNET.index,
                STAKE_SMT_TYPE_MAINNET.dep_type
            ),
            NetworkType::Testnet => cell_dep!(
                &STAKE_SMT_TYPE_TESTNET.tx_hash,
                STAKE_SMT_TYPE_TESTNET.index,
                STAKE_SMT_TYPE_TESTNET.dep_type
            ),
        }
    }

    pub async fn smt_cell_dep(ckb_rpc: &impl CkbRpc, type_id: &H256) -> Result<CellDep> {
        unique_cell_dep(ckb_rpc, Self::smt_type(type_id)).await
    }

    pub fn witness(mode: u8) -> WitnessArgs {
        let lock_field = StakeAtWitness::new_builder().mode(mode.into()).build();

        if mode == 0 { // staker unlock
             // todo: eth sig placeholder
        }

        WitnessArgs::new_builder()
            .lock(Some(lock_field.as_bytes()).pack())
            .build()
    }

    pub fn item(stake: &StakeInfoDelta) -> StakeItem {
        StakeItem {
            is_increase:        to_bool(&stake.is_increase()),
            amount:             to_u128(&stake.amount()),
            inauguration_epoch: to_u64(&stake.inauguration_epoch()),
        }
    }

    pub async fn get_cell(
        ckb_rpc: &impl CkbRpc,
        stake_lock: Script,
        xudt: Script,
    ) -> Result<Option<Cell>> {
        get_cell_by_scripts(ckb_rpc, stake_lock, xudt).await
    }

    pub async fn get_smt_cell(ckb_rpc: &impl CkbRpc, delegate_smt_type: Script) -> Result<Cell> {
        get_cell_by_type(ckb_rpc, delegate_smt_type).await
    }

    pub fn smt_witness(
        mode: u8,
        all_stake_infos: Vec<StakeInfo>,
        old_epoch_proof: Vec<u8>,
        new_epoch_proof: Vec<u8>,
    ) -> WitnessArgs {
        let type_field = AStakeSmtWitness::from(StakeSmtWitness {
            mode,
            update_info: StakeSmtUpdateInfo {
                all_stake_infos,
                old_epoch_proof,
                new_epoch_proof,
            },
        });
        WitnessArgs::new_builder()
            .input_type(Some(type_field.as_bytes()).pack())
            .build()
    }
}
