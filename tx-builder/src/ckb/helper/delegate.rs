use anyhow::Result;
use bytes::Bytes;
use ckb_types::packed::{CellDep, OutPoint, Script, WitnessArgs};
use ckb_types::prelude::{Builder, Entity, Pack};
use ckb_types::{H160, H256};

use common::traits::ckb_rpc_client::CkbRpc;
use common::types::axon_types::delegate::{
    DelegateArgs, DelegateAtWitness, DelegateInfoDelta, DelegateRequirementArgs,
    DelegateSmtWitness as ADelegateSmtWitness,
};
use common::types::ckb_rpc_client::Cell;
use common::types::tx_builder::{DelegateItem, NetworkType};
use common::utils::convert::*;

use crate::ckb::define::scripts::*;
use crate::ckb::define::types::{DelegateSmtUpdateInfo, DelegateSmtWitness, StakeGroupInfo};
use crate::ckb::helper::ckb::cell_collector::{get_cell_by_scripts, get_cell_by_type};
use crate::ckb::helper::metadata::Metadata;
use crate::ckb::helper::unique_cell_dep;
use crate::ckb::NETWORK_TYPE;
use crate::{cell_dep, out_point, script};

pub struct Delegate;

impl Delegate {
    pub fn lock(metadata_type_id: &H256, delegate_addr: &H160) -> Script {
        let metadata_type_hash = Metadata::type_(metadata_type_id).calc_script_hash();
        let args = DelegateArgs::new_builder()
            .metadata_type_id(to_axon_byte32(&metadata_type_hash))
            .delegator_addr(to_identity(delegate_addr))
            .build()
            .as_bytes();

        match **NETWORK_TYPE.load() {
            NetworkType::Mainnet => script!(
                &DELEGATE_LOCK_MAINNET.code_hash,
                DELEGATE_LOCK_MAINNET.hash_type,
                args
            ),
            NetworkType::Testnet => script!(
                &DELEGATE_LOCK_TESTNET.code_hash,
                DELEGATE_LOCK_TESTNET.hash_type,
                args
            ),
            NetworkType::Devnet => script!(
                &DELEGATE_LOCK_DEVNET.code_hash,
                DELEGATE_LOCK_DEVNET.hash_type,
                args
            ),
        }
    }

    pub fn smt_type(delegate_smt_type_id: &H256) -> Script {
        let args = Bytes::from(delegate_smt_type_id.as_bytes().to_vec());

        match **NETWORK_TYPE.load() {
            NetworkType::Mainnet => script!(
                &DELEGATE_SMT_TYPE_MAINNET.code_hash,
                DELEGATE_SMT_TYPE_MAINNET.hash_type,
                args
            ),
            NetworkType::Testnet => script!(
                &DELEGATE_SMT_TYPE_TESTNET.code_hash,
                DELEGATE_SMT_TYPE_TESTNET.hash_type,
                args
            ),
            NetworkType::Devnet => script!(
                &DELEGATE_SMT_TYPE_DEVNET.code_hash,
                DELEGATE_SMT_TYPE_DEVNET.hash_type,
                args
            ),
        }
    }

    pub fn requirement_type(metadata_type_id: &H256, requirement_type_id: &H256) -> Script {
        let metadata_type_hash = Metadata::type_(metadata_type_id).calc_script_hash();

        let args = DelegateRequirementArgs::new_builder()
            .metadata_type_id(to_axon_byte32(&metadata_type_hash))
            .requirement_type_id(to_byte32(requirement_type_id))
            .build()
            .as_bytes();

        match **NETWORK_TYPE.load() {
            NetworkType::Mainnet => script!(
                &DELEGATE_REQUIREMENT_TYPE_MAINNET.code_hash,
                DELEGATE_REQUIREMENT_TYPE_MAINNET.hash_type,
                args
            ),
            NetworkType::Testnet => script!(
                &DELEGATE_REQUIREMENT_TYPE_TESTNET.code_hash,
                DELEGATE_REQUIREMENT_TYPE_TESTNET.hash_type,
                args
            ),
            NetworkType::Devnet => script!(
                &DELEGATE_REQUIREMENT_TYPE_DEVNET.code_hash,
                DELEGATE_REQUIREMENT_TYPE_DEVNET.hash_type,
                args
            ),
        }
    }

    pub fn lock_dep() -> CellDep {
        match **NETWORK_TYPE.load() {
            NetworkType::Mainnet => cell_dep!(
                &DELEGATE_LOCK_MAINNET.tx_hash,
                DELEGATE_LOCK_MAINNET.index,
                DELEGATE_LOCK_MAINNET.dep_type
            ),
            NetworkType::Testnet => cell_dep!(
                &DELEGATE_LOCK_TESTNET.tx_hash,
                DELEGATE_LOCK_TESTNET.index,
                DELEGATE_LOCK_TESTNET.dep_type
            ),
            NetworkType::Devnet => cell_dep!(
                &DELEGATE_LOCK_DEVNET.tx_hash,
                DELEGATE_LOCK_DEVNET.index,
                DELEGATE_LOCK_DEVNET.dep_type
            ),
        }
    }

    pub fn smt_type_dep() -> CellDep {
        match **NETWORK_TYPE.load() {
            NetworkType::Mainnet => cell_dep!(
                &DELEGATE_SMT_TYPE_MAINNET.tx_hash,
                DELEGATE_SMT_TYPE_MAINNET.index,
                DELEGATE_SMT_TYPE_MAINNET.dep_type
            ),
            NetworkType::Testnet => cell_dep!(
                &DELEGATE_SMT_TYPE_TESTNET.tx_hash,
                DELEGATE_SMT_TYPE_TESTNET.index,
                DELEGATE_SMT_TYPE_TESTNET.dep_type
            ),
            NetworkType::Devnet => cell_dep!(
                &DELEGATE_SMT_TYPE_DEVNET.tx_hash,
                DELEGATE_SMT_TYPE_DEVNET.index,
                DELEGATE_SMT_TYPE_DEVNET.dep_type
            ),
        }
    }

    pub async fn smt_cell_dep(ckb_rpc: &impl CkbRpc, type_id: &H256) -> Result<CellDep> {
        unique_cell_dep(ckb_rpc, Self::smt_type(type_id)).await
    }

    pub fn requriement_type_dep() -> CellDep {
        match **NETWORK_TYPE.load() {
            NetworkType::Mainnet => cell_dep!(
                &DELEGATE_REQUIREMENT_TYPE_MAINNET.tx_hash,
                DELEGATE_REQUIREMENT_TYPE_MAINNET.index,
                DELEGATE_REQUIREMENT_TYPE_MAINNET.dep_type
            ),
            NetworkType::Testnet => cell_dep!(
                &DELEGATE_REQUIREMENT_TYPE_TESTNET.tx_hash,
                DELEGATE_REQUIREMENT_TYPE_TESTNET.index,
                DELEGATE_REQUIREMENT_TYPE_TESTNET.dep_type
            ),
            NetworkType::Devnet => cell_dep!(
                &DELEGATE_REQUIREMENT_TYPE_DEVNET.tx_hash,
                DELEGATE_REQUIREMENT_TYPE_DEVNET.index,
                DELEGATE_REQUIREMENT_TYPE_DEVNET.dep_type
            ),
        }
    }

    pub fn item(delegate: &DelegateInfoDelta) -> DelegateItem {
        DelegateItem {
            staker:             to_h160(&delegate.staker()),
            total_amount:       to_u128(&delegate.total_amount()),
            is_increase:        to_bool(&delegate.is_increase()),
            amount:             to_u128(&delegate.amount()),
            inauguration_epoch: to_u64(&delegate.inauguration_epoch()),
        }
    }

    pub async fn get_cell(
        ckb_rpc: &impl CkbRpc,
        delegate_lock: Script,
        xudt: Script,
    ) -> Result<Option<Cell>> {
        get_cell_by_scripts(ckb_rpc, delegate_lock, xudt).await
    }

    pub async fn get_requirement_cell(
        ckb_rpc: &impl CkbRpc,
        delegate_requirement_type: Script,
    ) -> Result<Cell> {
        get_cell_by_type(ckb_rpc, delegate_requirement_type).await
    }

    pub async fn get_smt_cell(ckb_rpc: &impl CkbRpc, delegate_smt_type: Script) -> Result<Cell> {
        get_cell_by_type(ckb_rpc, delegate_smt_type).await
    }

    pub fn witness(mode: u8) -> WitnessArgs {
        let lock_field = DelegateAtWitness::new_builder().mode(mode.into()).build();

        WitnessArgs::new_builder()
            .lock(Some(lock_field.as_bytes()).pack())
            .build()
    }

    pub fn smt_witness(mode: u8, all_stake_group_infos: Vec<StakeGroupInfo>) -> WitnessArgs {
        let type_field = ADelegateSmtWitness::from(DelegateSmtWitness {
            mode,
            update_info: DelegateSmtUpdateInfo {
                all_stake_group_infos,
            },
        });

        WitnessArgs::new_builder()
            .input_type(Some(type_field.as_bytes()).pack())
            .build()
    }
}
