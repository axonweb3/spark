use anyhow::Result;
use async_trait::async_trait;
use ckb_sdk::unlock::{InfoCellData, ScriptSigner};
use ckb_sdk::{ScriptGroup, ScriptGroupType};
use ckb_types::H160;
use ckb_types::{
    bytes::Bytes,
    core::{Capacity, TransactionBuilder, TransactionView},
    packed::{Byte32, CellOutput},
    prelude::{Entity, Pack},
    H256,
};
use molecule::prelude::Builder;

use common::traits::ckb_rpc_client::CkbRpc;
use common::traits::tx_builder::IInitTxBuilder;
use common::types::axon_types::{
    checkpoint::CheckpointCellData, delegate::DelegateSmtCellData as ADelegateSmtCellData,
    metadata::MetadataCellData as AMetadataCellData,
    reward::RewardSmtCellData as ARewardSmtCellData, stake::StakeSmtCellData as AStakeSmtCellData,
};
use common::types::tx_builder::*;
use common::utils::convert::{to_axon_byte32, to_h256};

use crate::ckb::define::constants::START_EPOCH;
use crate::ckb::define::scripts::*;
use crate::ckb::define::types::{
    DelegateSmtCellData, MetadataCellData, RewardSmtCellData, StakeSmtCellData,
};
use crate::ckb::helper::{
    AlwaysSuccess, Checkpoint as HCheckpoint, Delegate, Metadata as HMetadata, OmniEth, Reward,
    Secp256k1, Selection, Stake, Tx, TypeId, Xudt,
};
use crate::ckb::NETWORK_TYPE;

pub struct InitTxBuilder<'a, C: CkbRpc> {
    ckb:        &'a C,
    seeder_key: PrivateKey,
    max_supply: Amount,
    checkpoint: Checkpoint,
    metadata:   Metadata,
}

#[async_trait]
impl<'a, C: CkbRpc> IInitTxBuilder<'a, C> for InitTxBuilder<'a, C> {
    fn new(
        ckb: &'a C,
        seeder_key: PrivateKey,
        max_supply: Amount,
        checkpoint: Checkpoint,
        metadata: Metadata,
    ) -> Self {
        Self {
            ckb,
            seeder_key,
            max_supply,
            checkpoint,
            metadata,
        }
    }

    async fn build_tx(&self) -> Result<(TransactionView, TypeIds)> {
        let omni_eth = OmniEth::new(self.seeder_key.clone());
        let seeder_lock = OmniEth::lock(&omni_eth.address()?);

        let outputs_data = self.build_data();

        let outputs = vec![
            // issue cell
            CellOutput::new_builder()
                .lock(OmniEth::supply_lock(H160::default(), Byte32::default())?)
                .type_(Some(TypeId::mock()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[0].len())?)?,
            // selection cell
            CellOutput::new_builder()
                .lock(Selection::lock(&Byte32::default(), &Byte32::default()))
                .type_(Some(TypeId::mock()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[1].len())?)?,
            // checkpoint cell
            CellOutput::new_builder()
                .lock(AlwaysSuccess::lock())
                .type_(Some(HCheckpoint::type_(&H256::default())).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[2].len())?)?,
            // metadata cell
            CellOutput::new_builder()
                .lock(AlwaysSuccess::lock())
                .type_(Some(HMetadata::type_(&H256::default())).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[3].len())?)?,
            // stake smt cell
            CellOutput::new_builder()
                .lock(AlwaysSuccess::lock())
                .type_(Some(Stake::smt_type(&H256::default())).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[4].len())?)?,
            // delegate smt cell
            CellOutput::new_builder()
                .lock(AlwaysSuccess::lock())
                .type_(Some(Delegate::smt_type(&H256::default())).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[5].len())?)?,
            // reward smt cell
            CellOutput::new_builder()
                .lock(AlwaysSuccess::lock())
                .type_(Some(TypeId::mock()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[6].len())?)?,
        ];

        let cell_deps = vec![
            OmniEth::lock_dep(),
            Secp256k1::lock_dep(),
            HCheckpoint::type_dep(),
            HMetadata::type_dep(),
            Stake::smt_type_dep(),
            Delegate::smt_type_dep(),
            Reward::smt_type_dep(),
        ];

        let witnesses = vec![
            OmniEth::witness_placeholder().as_bytes(), // capacity provider lock
        ];

        let tx = TransactionBuilder::default()
            .inputs(vec![])
            .outputs(outputs)
            .outputs_data(outputs_data.pack())
            .cell_deps(cell_deps)
            .witnesses(witnesses.pack())
            .build();

        let tx = Tx::new(self.ckb, tx).balance(seeder_lock.clone()).await?;

        let (tx, type_id_args) = self.modify_outputs(tx, omni_eth.address()?)?;

        let signer = omni_eth.signer()?;
        let tx = signer.sign_tx(&tx, &ScriptGroup {
            script:         seeder_lock,
            group_type:     ScriptGroupType::Lock,
            input_indices:  vec![0],
            output_indices: vec![],
        })?;

        Ok((tx, type_id_args))
    }
}

impl<'a, C: CkbRpc> InitTxBuilder<'a, C> {
    fn build_data(&self) -> Vec<Bytes> {
        vec![
            // issue cell data
            InfoCellData::new_simple(0, 0, H256::default()).pack(),
            // selection cell data
            Bytes::default(),
            // checkpoint cell data
            CheckpointCellData::new_builder().build().as_bytes(),
            // metadata cell data
            AMetadataCellData::from(MetadataCellData {
                metadata: vec![self.metadata.clone()],
                ..Default::default()
            })
            .as_bytes(),
            // stake smt cell data
            AStakeSmtCellData::default().as_bytes(),
            // delegate smt cell data
            ADelegateSmtCellData::default().as_bytes(),
            // reward smt cell data
            ARewardSmtCellData::default().as_bytes(),
        ]
    }

    fn modify_outputs(
        &self,
        tx: TransactionView,
        seeder_addr: H160,
    ) -> Result<(TransactionView, TypeIds)> {
        let mut outputs = tx.outputs().into_iter().collect::<Vec<_>>();
        let mut outputs_data = tx.outputs_data().into_iter().collect::<Vec<_>>();

        let first_input = tx.inputs().get(0).unwrap();

        // issue cell
        let issue_type_id = TypeId::calc(&first_input, 0);
        let issue_type = TypeId::script(&issue_type_id);
        let issue_lock = OmniEth::supply_lock(seeder_addr, issue_type.calc_script_hash())?;
        let issue_lock_hash = issue_lock.calc_script_hash();
        outputs[0] = tx
            .output(0)
            .unwrap()
            .as_builder()
            .lock(issue_lock)
            .type_(Some(issue_type).pack())
            .build();

        // checkpoint cell
        let checkpoint_type_id = TypeId::calc(&first_input, 2);
        outputs[2] = tx
            .output(2)
            .unwrap()
            .as_builder()
            .type_(Some(HCheckpoint::type_(&checkpoint_type_id)).pack())
            .build();

        // metadata cell
        let metadata_type_id = TypeId::calc(&first_input, 3);
        outputs[3] = tx
            .output(3)
            .unwrap()
            .as_builder()
            .type_(Some(HMetadata::type_(&metadata_type_id)).pack())
            .build();

        // stake smt cell
        let stake_smt_type_id = TypeId::calc(&first_input, 4);
        outputs[4] = tx
            .output(4)
            .unwrap()
            .as_builder()
            .type_(Some(Stake::smt_type(&stake_smt_type_id)).pack())
            .build();

        // delegate smt cell
        let delegate_smt_type_id = TypeId::calc(&first_input, 5);
        outputs[5] = tx
            .output(5)
            .unwrap()
            .as_builder()
            .type_(Some(Delegate::smt_type(&delegate_smt_type_id)).pack())
            .build();

        // reward smt cell
        let reward_smt_type_id = TypeId::calc(&first_input, 6);
        let reward_type = TypeId::script(&reward_smt_type_id); // todo
        let reward_type_hash = reward_type.calc_script_hash();
        outputs[6] = tx
            .output(6)
            .unwrap()
            .as_builder()
            .type_(Some(reward_type).pack())
            .build();

        // selection cell
        let selection_type_id = TypeId::calc(&first_input, 1);
        let selection_type = TypeId::script(&selection_type_id);
        let selection_lock = Selection::lock(&issue_lock_hash, &reward_type_hash);
        let selection_lock_hash = selection_lock.calc_script_hash();
        outputs[1] = tx
            .output(1)
            .unwrap()
            .as_builder()
            .lock(selection_lock)
            .type_(Some(selection_type).pack())
            .build();

        // issue cell data
        outputs_data[0] = InfoCellData::new_simple(
            0,
            self.max_supply,
            to_h256(&Xudt::type_(&selection_lock_hash).calc_script_hash()),
        )
        .pack()
        .pack();

        // checkpoint cell data
        let checkpoint: CheckpointCellData = self.checkpoint.clone().into();
        outputs_data[2] = checkpoint
            .as_builder()
            .metadata_type_id(to_axon_byte32(
                &HMetadata::type_(&metadata_type_id).calc_script_hash(),
            ))
            .build()
            .as_bytes()
            .pack();

        let network_type = NETWORK_TYPE.load();

        let type_ids = TypeIds {
            issue_type_id,
            selection_type_id,
            checkpoint_type_id,
            metadata_type_id: metadata_type_id.clone(),
            reward_smt_type_id,
            stake_smt_type_id,
            delegate_smt_type_id,
            xudt_owner: to_h256(&selection_lock_hash),
            checkpoint_code_hash: if **network_type == NetworkType::Mainnet {
                CHECKPOINT_TYPE_MAINNET.code_hash.clone()
            } else {
                CHECKPOINT_TYPE_TESTNET.code_hash.clone()
            },
            metadata_code_hash: if **network_type == NetworkType::Mainnet {
                METADATA_TYPE_MAINNET.code_hash.clone()
            } else {
                METADATA_TYPE_TESTNET.code_hash.clone()
            },
            reward_code_hash: if **network_type == NetworkType::Mainnet {
                REWARD_SMT_TYPE_MAINNET.code_hash.clone()
            } else {
                REWARD_SMT_TYPE_TESTNET.code_hash.clone()
            },
            stake_code_hash: if **network_type == NetworkType::Mainnet {
                STAKE_SMT_TYPE_MAINNET.code_hash.clone()
            } else {
                STAKE_SMT_TYPE_TESTNET.code_hash.clone()
            },
            delegate_code_hash: if **network_type == NetworkType::Mainnet {
                DELEGATE_SMT_TYPE_MAINNET.code_hash.clone()
            } else {
                DELEGATE_SMT_TYPE_TESTNET.code_hash.clone()
            },
            withdraw_code_hash: if **network_type == NetworkType::Mainnet {
                WITHDRAW_LOCK_MAINNET.code_hash.clone()
            } else {
                WITHDRAW_LOCK_TESTNET.code_hash.clone()
            },
            xudt_type_hash: to_h256(&Xudt::type_(&selection_lock_hash).calc_script_hash()),
        };

        // metadata cell data
        outputs_data[3] = AMetadataCellData::from(MetadataCellData {
            epoch:                  START_EPOCH,
            propose_count_smt_root: H256::default(),
            metadata:               vec![self.metadata.clone()],
            type_ids:               type_ids.clone(),
        })
        .as_bytes()
        .pack();

        // stake smt cell data
        outputs_data[4] = AStakeSmtCellData::from(StakeSmtCellData {
            metadata_type_id: metadata_type_id.clone(),
            ..Default::default()
        })
        .as_bytes()
        .pack();

        // delegate smt cell data
        outputs_data[5] = ADelegateSmtCellData::from(DelegateSmtCellData {
            metadata_type_id: metadata_type_id.clone(),
            ..Default::default()
        })
        .as_bytes()
        .pack();

        // reward smt cell data
        outputs_data[6] = ARewardSmtCellData::from(RewardSmtCellData {
            metadata_type_id,
            ..Default::default()
        })
        .as_bytes()
        .pack();

        let tx = tx
            .as_advanced_builder()
            .set_outputs(outputs)
            .set_outputs_data(outputs_data)
            .build();

        Ok((tx, type_ids))
    }
}
