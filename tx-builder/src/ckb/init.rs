use anyhow::Result;
use async_trait::async_trait;
use axon_types::{
    checkpoint::CheckpointCellData, delegate::DelegateSmtCellData,
    metadata::MetadataCellData as AMetadataCellData, reward::RewardSmtCellData,
    stake::StakeSmtCellData,
};
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
use common::types::tx_builder::*;
use common::utils::convert::to_h256;

use crate::ckb::define::constants::START_EPOCH;
use crate::ckb::define::scripts::*;
use crate::ckb::define::types::MetadataCellData;
use crate::ckb::utils::cell_dep::*;
use crate::ckb::utils::omni::*;
use crate::ckb::utils::script::*;
use crate::ckb::utils::tx::balance_tx;

pub struct InitTxBuilder<C: CkbRpc> {
    ckb:        CkbNetwork<C>,
    seeder_key: PrivateKey,
    max_supply: Amount,
    checkpoint: Checkpoint,
    metadata:   Metadata,
}

#[async_trait]
impl<C: CkbRpc> IInitTxBuilder<C> for InitTxBuilder<C> {
    fn new(
        ckb: CkbNetwork<C>,
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
        let seeder_addr = omni_eth_address(self.seeder_key.clone())?;
        let seeder_lock = omni_eth_lock(&self.ckb.network_type, &seeder_addr);

        let outputs_data = self.build_data();

        let outputs = vec![
            // issue cell
            CellOutput::new_builder()
                .lock(omni_eth_supply_lock(
                    &self.ckb.network_type,
                    H160::default(),
                    Byte32::default(),
                )?)
                .type_(Some(default_type_id()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[0].len())?)?,
            // selection cell
            CellOutput::new_builder()
                .lock(selection_lock(
                    &self.ckb.network_type,
                    &Byte32::default(),
                    &Byte32::default(),
                ))
                .type_(Some(default_type_id()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[1].len())?)?,
            // checkpoint cell
            CellOutput::new_builder()
                .lock(always_success_lock(&self.ckb.network_type))
                .type_(Some(checkpoint_type(&self.ckb.network_type, &H256::default())).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[2].len())?)?,
            // metadata cell
            CellOutput::new_builder()
                .lock(always_success_lock(&self.ckb.network_type))
                .type_(Some(metadata_type(&self.ckb.network_type, &H256::default())).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[3].len())?)?,
            // stake smt cell
            CellOutput::new_builder()
                .lock(always_success_lock(&self.ckb.network_type))
                .type_(Some(default_type_id()).pack())
                // .type_(Some(stake_smt_type(&self.ckb.network_type, &H256::default())).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[4].len())?)?,
            // delegate smt cell
            CellOutput::new_builder()
                .lock(always_success_lock(&self.ckb.network_type))
                .type_(Some(default_type_id()).pack())
                // .type_(Some(delegate_type(&self.ckb.network_type, &H256::default())).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[5].len())?)?,
            // reward smt cell
            CellOutput::new_builder()
                .lock(always_success_lock(&self.ckb.network_type))
                .type_(Some(default_type_id()).pack())
                // .type_(Some(reward_type(&self.ckb.network_type, &H256::default())).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[6].len())?)?,
        ];

        let cell_deps = vec![
            omni_lock_dep(&self.ckb.network_type),
            secp256k1_lock_dep(&self.ckb.network_type),
            checkpoint_type_dep(&self.ckb.network_type),
            metadata_type_dep(&self.ckb.network_type),
            // stake_smt_type_dep(&self.ckb.network_type),
            // delegate_smt_type_dep(&self.ckb.network_type),
            // reward_type_dep(&self.ckb.network_type),
        ];

        let witnesses = vec![
            omni_eth_witness_placeholder().as_bytes(), // capacity provider lock
        ];

        let tx = TransactionBuilder::default()
            .inputs(vec![])
            .outputs(outputs)
            .outputs_data(outputs_data.pack())
            .cell_deps(cell_deps)
            .witnesses(witnesses.pack())
            .build();

        let tx = balance_tx(&self.ckb.client, seeder_lock.clone(), tx).await?;

        let (tx, type_id_args) = self.modify_outputs(tx, seeder_addr)?;

        let signer = omni_eth_signer(self.seeder_key.clone())?;
        let tx = signer.sign_tx(&tx, &ScriptGroup {
            script:         seeder_lock,
            group_type:     ScriptGroupType::Lock,
            input_indices:  vec![0],
            output_indices: vec![0],
        })?;

        Ok((tx, type_id_args))
    }
}

impl<C: CkbRpc> InitTxBuilder<C> {
    fn build_data(&self) -> Vec<Bytes> {
        let checkpoint: CheckpointCellData = (&self.checkpoint).into();
        vec![
            // issue cell data
            InfoCellData::new_simple(0, 0, H256::default()).pack(),
            // selection cell data
            Bytes::default(),
            // checkpoint cell data
            checkpoint.as_bytes(),
            // metadata cell data
            AMetadataCellData::from(MetadataCellData {
                metadata: vec![self.metadata.clone()],
                ..Default::default()
            })
            .as_bytes(),
            // stake smt cell data
            StakeSmtCellData::default().as_bytes(),
            // delegate smt cell data
            DelegateSmtCellData::default().as_bytes(),
            // reward smt cell data
            RewardSmtCellData::default().as_bytes(),
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
        let issue_type_id = type_id(&first_input, 0);
        let issue_type = type_id_script(&issue_type_id);
        let issue_lock = omni_eth_supply_lock(
            &self.ckb.network_type,
            seeder_addr,
            issue_type.calc_script_hash(),
        )?;
        let issue_lock_hash = issue_lock.calc_script_hash();
        outputs[0] = tx
            .output(0)
            .unwrap()
            .as_builder()
            .lock(issue_lock)
            .type_(Some(issue_type).pack())
            .build();

        // checkpoint cell
        let checkpoint_type_id = type_id(&first_input, 2);
        outputs[2] = tx
            .output(2)
            .unwrap()
            .as_builder()
            .type_(Some(checkpoint_type(&self.ckb.network_type, &checkpoint_type_id)).pack())
            .build();

        // metadata cell
        let metadata_type_id = type_id(&first_input, 3);
        outputs[3] = tx
            .output(3)
            .unwrap()
            .as_builder()
            .type_(Some(metadata_type(&self.ckb.network_type, &metadata_type_id)).pack())
            .build();

        // stake smt cell
        let stake_smt_type_id = type_id(&first_input, 4);
        outputs[4] = tx
            .output(4)
            .unwrap()
            .as_builder()
            .type_(Some(type_id_script(&stake_smt_type_id)).pack())
            // .type_(Some(stake_smt_type(&self.ckb.network_type, &stake_smt_type_id)).pack())
            .build();

        // delegate smt cell
        let delegate_smt_type_id = type_id(&first_input, 5);
        outputs[5] = tx
            .output(5)
            .unwrap()
            .as_builder()
            // .type_(Some(delegate_type(&self.ckb.network_type, &metadata_type_id)).pack())
            .type_(Some(type_id_script(&delegate_smt_type_id)).pack())
            .build();

        // reward smt cell
        let reward_smt_type_id = type_id(&first_input, 6);
        let reward_type = type_id_script(&reward_smt_type_id); // todo
        let reward_type_hash = reward_type.calc_script_hash();
        outputs[6] = tx
            .output(6)
            .unwrap()
            .as_builder()
            .type_(Some(reward_type).pack())
            .build();

        // selection cell
        let selection_type_id = type_id(&first_input, 1);
        let selection_type = type_id_script(&selection_type_id);
        let selection_lock =
            selection_lock(&self.ckb.network_type, &issue_lock_hash, &reward_type_hash);
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
            to_h256(&xudt_type(&self.ckb.network_type, &selection_lock_hash).calc_script_hash()),
        )
        .pack()
        .pack();

        let type_ids = TypeIds {
            issue_type_id,
            selection_type_id,
            checkpoint_type_id,
            metadata_type_id,
            reward_smt_type_id,
            stake_smt_type_id,
            delegate_smt_type_id,
            xudt_owner: to_h256(&selection_lock_hash),
            checkpoint_code_hash: if self.ckb.network_type == NetworkType::Mainnet {
                CHECKPOINT_TYPE_MAINNET.code_hash.clone()
            } else {
                CHECKPOINT_TYPE_TESTNET.code_hash.clone()
            },
            metadata_code_hash: if self.ckb.network_type == NetworkType::Mainnet {
                METADATA_TYPE_MAINNET.code_hash.clone()
            } else {
                METADATA_TYPE_TESTNET.code_hash.clone()
            },
            reward_code_hash: if self.ckb.network_type == NetworkType::Mainnet {
                REWARD_TYPE_MAINNET.code_hash.clone()
            } else {
                REWARD_TYPE_TESTNET.code_hash.clone()
            },
            stake_code_hash: if self.ckb.network_type == NetworkType::Mainnet {
                STAKE_SMT_TYPE_MAINNET.code_hash.clone()
            } else {
                STAKE_SMT_TYPE_TESTNET.code_hash.clone()
            },
            delegate_code_hash: if self.ckb.network_type == NetworkType::Mainnet {
                DELEGATE_SMT_TYPE_MAINNET.code_hash.clone()
            } else {
                DELEGATE_SMT_TYPE_TESTNET.code_hash.clone()
            },
            withdraw_code_hash: if self.ckb.network_type == NetworkType::Mainnet {
                WITHDRAW_LOCK_MAINNET.code_hash.clone()
            } else {
                WITHDRAW_LOCK_TESTNET.code_hash.clone()
            },
            xudt_type_hash: to_h256(
                &xudt_type(&self.ckb.network_type, &selection_lock_hash).calc_script_hash(),
            ),
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

        let tx = tx
            .as_advanced_builder()
            .set_outputs(outputs)
            .set_outputs_data(outputs_data)
            .build();

        Ok((tx, type_ids))
    }
}
