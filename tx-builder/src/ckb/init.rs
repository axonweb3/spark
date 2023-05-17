use anyhow::Result;
use async_trait::async_trait;
use axon_types::checkpoint::CheckpointCellData;
use ckb_sdk::unlock::ScriptSigner;
use ckb_sdk::{ScriptGroup, ScriptGroupType};
use ckb_types::{
    bytes::Bytes,
    core::{Capacity, TransactionBuilder, TransactionView},
    packed::CellOutput,
    prelude::{Entity, Pack},
};

use common::traits::ckb_rpc_client::CkbRpc;
use common::traits::tx_builder::IInitTxBuilder;
use common::types::tx_builder::*;

use crate::ckb::utils::cell_dep::*;
use crate::ckb::utils::omni::*;
use crate::ckb::utils::script::*;
use crate::ckb::utils::tx::balance_tx;

pub struct InitTxBuilder<C: CkbRpc> {
    ckb_client:   C,
    network_type: NetworkType,
    kicker_key:   PrivateKey,
    _scripts:     Scripts,
    checkpoint:   Checkpoint,
    _metadata:    Metadata,
}

#[async_trait]
impl<C: CkbRpc> IInitTxBuilder<C> for InitTxBuilder<C> {
    fn new(
        ckb_client: C,
        network_type: NetworkType,
        kicker_key: PrivateKey,
        _scripts: Scripts,
        checkpoint: Checkpoint,
        _metadata: Metadata,
    ) -> Self {
        Self {
            ckb_client,
            network_type,
            kicker_key,
            _scripts,
            checkpoint,
            _metadata,
        }
    }

    async fn build_tx(&self) -> Result<TransactionView> {
        let kicker = omni_eth_address(self.kicker_key.clone())?;
        let kicker_lock = omni_eth_lock(&self.network_type, &kicker);

        let outputs_data = self.build_data();

        let outputs = vec![
            // todo:
            // selection cell
            // CellOutput::new_builder()
            //     // todo: metadata_type_id | reward_smt_type_id
            //     .lock(selection_lock(
            //          &self.scripts.selection_lock_code_hash, H256::default(), H256::default()))
            //     .type_(None.pack())
            //     .build_exact_capacity(Capacity::bytes(outputs_data[0].len())?)?,
            // // issue cell
            // CellOutput::new_builder()
            //     .lock(fake_lock.clone()) // omni lock
            //     .build_exact_capacity(Capacity::bytes(outputs_data[1].len())?)?,
            // // checkpoint cell
            // CellOutput::new_builder()
            //     .lock(cannot_destroy_lock(&self.network_type))
            //     .build_exact_capacity(Capacity::bytes(outputs_data[2].len())?)?,
            // // metadata cell
            // CellOutput::new_builder()
            //     .lock(cannot_destroy_lock(&self.network_type))
            //     .build_exact_capacity(Capacity::bytes(outputs_data[3].len())?)?,
            // // stake smt cell
            // CellOutput::new_builder()
            //     .lock(cannot_destroy_lock(&self.network_type))
            //     .build_exact_capacity(Capacity::bytes(outputs_data[4].len())?)?,
            // // delegate smt cell
            // CellOutput::new_builder()
            //     .lock(cannot_destroy_lock(&self.network_type))
            //     .build_exact_capacity(Capacity::bytes(outputs_data[5].len())?)?,
            // // reward smt cell
            // CellOutput::new_builder()
            //     .lock(cannot_destroy_lock(&self.network_type))
            //     .build_exact_capacity(Capacity::bytes(outputs_data[6].len())?)?,
            // // AT cell
            // CellOutput::new_builder()
            //     .lock(fake_lock.clone())
            //     .type_(Some(fake_type.clone()).pack())
            //     .build_exact_capacity(Capacity::bytes(outputs_data[8].len())?)?,
            // Ckb cell
            CellOutput::new_builder()
                .lock(kicker_lock.clone())
                .build_exact_capacity(Capacity::zero())
                .unwrap(),
        ];

        let cell_deps = vec![
            omni_lock_dep(&self.network_type),
            secp256k1_lock_dep(&self.network_type),
            // xudt_dep(&self.network_type),
        ];

        let witnesses = vec![omni_eth_witness_placeholder().as_bytes()];

        let tx = TransactionBuilder::default()
            .inputs(vec![])
            .outputs(outputs)
            .outputs_data(outputs_data.pack())
            .cell_deps(cell_deps)
            .witnesses(witnesses.pack())
            .build();

        let tx = balance_tx(&self.ckb_client, kicker_lock.clone(), tx).await?;

        let signer = omni_eth_signer(self.kicker_key.clone())?;
        let tx = signer.sign_tx(&tx, &ScriptGroup {
            script:         kicker_lock,
            group_type:     ScriptGroupType::Lock,
            input_indices:  vec![0],
            output_indices: vec![0],
        })?;

        Ok(tx)
    }
}

impl<C: CkbRpc> InitTxBuilder<C> {
    fn build_data(&self) -> Vec<Bytes> {
        let _checkpoint: CheckpointCellData = (&self.checkpoint).into();
        vec![
            // selection cell data
            // Bytes::default(),
            // todo:
            // // issue cell data
            // Bytes::new(), // todo
            // // checkpoint cell data
            // checkpoint.as_bytes(),
            // // metadata cell data
            // metadata_cell_data(
            //     START_EPOCH,
            //     self.type_ids.clone(),
            //     &vec![self.metadata.clone()],
            //     Byte32::default(),
            // ).as_bytes(),
            // // stake smt cell data
            // stake_smt_cell_data(Byte32::default()).as_bytes(),
            // // delegate smt cell data
            // delegate_smt_cell_data(vec![]).as_bytes(),
            // // reward smt cell data
            // reward_smt_cell_data(Byte32::default()).as_bytes(),
            // // AT cell data
            // Bytes::default(), // todo
            // Ckb cell data
            Bytes::default(),
        ]
    }
}
