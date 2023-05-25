use anyhow::Result;
use async_trait::async_trait;
use axon_types::checkpoint::CheckpointCellData;
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

// use crate::ckb::define::config::START_EPOCH;
// use crate::ckb::utils::cell_data::*;
use crate::ckb::utils::cell_dep::*;
use crate::ckb::utils::omni::*;
use crate::ckb::utils::script::*;
use crate::ckb::utils::tx::balance_tx;

pub struct InitTxBuilder<C: CkbRpc> {
    ckb_client:   C,
    network_type: NetworkType,
    seeder_key:   PrivateKey,
    max_supply:   Amount,
    checkpoint:   Checkpoint,
    _metadata:    Metadata,
    _type_ids:    TypeIds,
}

#[async_trait]
impl<C: CkbRpc> IInitTxBuilder<C> for InitTxBuilder<C> {
    fn new(
        ckb_client: C,
        network_type: NetworkType,
        seeder_key: PrivateKey,
        max_supply: Amount,
        checkpoint: Checkpoint,
        _metadata: Metadata,
        _type_ids: TypeIds,
    ) -> Self {
        Self {
            ckb_client,
            network_type,
            seeder_key,
            max_supply,
            checkpoint,
            _metadata,
            _type_ids,
        }
    }

    async fn build_tx(&self) -> Result<(TransactionView, TypeIds)> {
        let seeder_addr = omni_eth_address(self.seeder_key.clone())?;
        let seeder_lock = omni_eth_lock(&self.network_type, &seeder_addr);

        let outputs_data = self.build_data();

        let outputs = vec![
            // issue cell
            CellOutput::new_builder()
                .lock(omni_eth_supply_lock(
                    &self.network_type,
                    H160::default(),
                    Byte32::default(),
                )?)
                .type_(Some(default_type_id()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[0].len())?)?,
            // selection cell
            CellOutput::new_builder()
                .lock(selection_lock(
                    &self.network_type,
                    &Byte32::default(),
                    &Byte32::default(),
                ))
                .type_(Some(default_type_id()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[1].len())?)?,
            // // checkpoint cell
            // CellOutput::new_builder()
            //     .lock(always_success_lock(&self.network_type))
            //     .type_(Some(checkpoint_type(&self.network_type, &H256::default())).pack())
            //     .build_exact_capacity(Capacity::bytes(outputs_data[2].len())?)?,
            // // metadata cell
            // CellOutput::new_builder()
            //     .lock(always_success_lock(&self.network_type))
            //     .type_(Some(metadata_type(&self.network_type, &H256::default())).pack())
            //     .build_exact_capacity(Capacity::bytes(outputs_data[3].len())?)?,
            // // stake smt cell
            // CellOutput::new_builder()
            //     .lock(always_success_lock(&self.network_type))
            //     .build_exact_capacity(Capacity::bytes(outputs_data[4].len())?)?,
            // // delegate smt cell
            // CellOutput::new_builder()
            //     .lock(always_success_lock(&self.network_type))
            //     .build_exact_capacity(Capacity::bytes(outputs_data[5].len())?)?,
            // // reward smt cell
            // CellOutput::new_builder()
            //     .lock(always_success_lock(&self.network_type))
            //     .build_exact_capacity(Capacity::bytes(outputs_data[6].len())?)?,
            // ckb cell
            CellOutput::new_builder()
                .lock(seeder_lock.clone())
                .build_exact_capacity(Capacity::zero())
                .unwrap(),
        ];

        let cell_deps = vec![
            omni_lock_dep(&self.network_type),
            secp256k1_lock_dep(&self.network_type),
            // checkpoint_dep(&self.network_type),
            // metadata_dep(&self.network_type),
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

        let tx = balance_tx(&self.ckb_client, seeder_lock.clone(), tx).await?;

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
        let _checkpoint: CheckpointCellData = (&self.checkpoint).into();
        vec![
            // issue cell data
            InfoCellData::new_simple(0, 0, H256::default()).pack(),
            // selection cell data
            Bytes::default(),
            // // checkpoint cell data
            // checkpoint.as_bytes(),
            // // metadata cell data
            // metadata_cell_data(
            //     START_EPOCH,
            //     self.type_ids.clone(),
            //     &[self.metadata.clone()],
            //     axon_types::basic::Byte32::default(),
            // )
            // .as_bytes(),
            // // stake smt cell data
            // stake_smt_cell_data(Byte32::default()).as_bytes(),
            // // delegate smt cell data
            // delegate_smt_cell_data(vec![]).as_bytes(),
            // // reward smt cell data
            // reward_smt_cell_data(Byte32::default()).as_bytes(),
            // Ckb cell data
            Bytes::default(),
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
        let issue_type_args = type_id(&first_input, 0);
        let issue_type_id = type_id_script(&issue_type_args);
        let issue_lock = omni_eth_supply_lock(
            &self.network_type,
            seeder_addr,
            issue_type_id.calc_script_hash(),
        )?;
        let issue_lock_hash = issue_lock.calc_script_hash();
        outputs[0] = tx
            .output(0)
            .unwrap()
            .as_builder()
            .lock(issue_lock)
            .type_(Some(issue_type_id).pack())
            .build();

        // selection cell
        let selection_type_args = type_id(&first_input, 1);
        let selection_type_id = type_id_script(&selection_type_args);
        let selection_lock = selection_lock(
            &self.network_type,
            &issue_lock_hash,
            &Byte32::default(), // todo: reward smt type id
        );
        let selection_lock_hash = selection_lock.calc_script_hash();
        outputs[1] = tx
            .output(1)
            .unwrap()
            .as_builder()
            .lock(selection_lock)
            .type_(Some(selection_type_id).pack())
            .build();

        // issue cell data
        outputs_data[0] = InfoCellData::new_simple(
            0,
            self.max_supply,
            to_h256(&xudt_type(&self.network_type, &selection_lock_hash).calc_script_hash()),
        )
        .pack()
        .pack();

        // // checkpoint cell
        // let checkpoint_type_args = type_id(&first_input, 2);
        // let checkpoint_type =
        //      checkpoint_type(&self.network_type, &checkpoint_type_args);
        // outputs[2] = tx
        //     .output(2)
        //     .unwrap()
        //     .as_builder()
        //     .type_(Some(checkpoint_type).pack())
        //     .build();

        // // metadata cell
        // let metadata_type_args = type_id(&first_input, 3);
        // let metadata_type = metadata_type(&self.network_type, &metadata_type_args);
        // outputs[3] = tx
        //     .output(3)
        //     .unwrap()
        //     .as_builder()
        //     .type_(Some(metadata_type).pack())
        //     .build();

        let tx = tx
            .as_advanced_builder()
            .set_outputs(outputs)
            .set_outputs_data(outputs_data)
            .build();

        let type_ids = TypeIds {
            issue_type_id:        issue_type_args,
            selection_type_id:    selection_type_args,
            checkpoint_type_id:   H256::default(),
            metadata_type_id:     H256::default(),
            reward_type_id:       H256::default(),
            stake_smt_type_id:    H256::default(),
            delegate_smt_type_id: H256::default(),
            xudt_lock_id:         to_h256(&selection_lock_hash),
        };

        Ok((tx, type_ids))
    }
}
