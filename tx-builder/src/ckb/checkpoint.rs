use anyhow::Result;
use async_trait::async_trait;
use axon_types::checkpoint::CheckpointCellData;
use bytes::Bytes;
use ckb_sdk::{unlock::ScriptSigner, ScriptGroup, ScriptGroupType};
use ckb_types::{
    core::{Capacity, TransactionBuilder, TransactionView},
    packed::{CellInput, CellOutput},
    prelude::{Entity, Pack},
};
use common::{
    traits::{ckb_rpc_client::CkbRpc, tx_builder::ICheckpointTxBuilder},
    types::tx_builder::{CheckpointTypeIds, CkbNetwork},
};
use common::{
    types::tx_builder::{Checkpoint, PrivateKey},
    utils::convert::{to_u32, to_u64},
};
use molecule::prelude::Builder;

use crate::ckb::{
    define::error::CkbTxErr,
    utils::{
        cell_collector::get_unique_cell,
        cell_dep::{
            always_success_lock_dep, checkpoint_type_dep, metadata_cell_dep, omni_lock_dep,
            secp256k1_lock_dep, xudt_type_dep,
        },
        omni::{omni_eth_address, omni_eth_signer, omni_eth_witness_placeholder},
        script::{always_success_lock, checkpoint_type, omni_eth_lock},
        tx::balance_tx,
    },
};

pub struct CheckpointTxBuilder<C>
where
    C: CkbRpc,
{
    kicker_key:     PrivateKey,
    ckb:            CkbNetwork<C>,
    type_ids:       CheckpointTypeIds,
    epoch_len:      u64,
    new_checkpoint: Checkpoint,
}

#[async_trait]
impl<C> ICheckpointTxBuilder<C> for CheckpointTxBuilder<C>
where
    C: CkbRpc,
{
    async fn new(
        kicker_key: PrivateKey,
        ckb: CkbNetwork<C>,
        type_ids: CheckpointTypeIds,
        epoch_len: u64,
        new_checkpoint: Checkpoint,
    ) -> Self {
        Self {
            kicker_key,
            ckb,
            type_ids,
            epoch_len,
            new_checkpoint,
        }
    }

    async fn build_tx(&self) -> Result<TransactionView> {
        let checkpoint_type =
            checkpoint_type(&self.ckb.network_type, &self.type_ids.checkpoint_type_id);

        let last_checkpoint_cell =
            get_unique_cell(&self.ckb.client, checkpoint_type.clone()).await?;

        let last_checkpoint_data = CheckpointCellData::new_unchecked(
            last_checkpoint_cell.output_data.unwrap().into_bytes(),
        );

        self.check_occasion(
            to_u64(&last_checkpoint_data.epoch()),
            to_u32(&last_checkpoint_data.period()),
        )
        .await?;

        let inputs: Vec<ckb_types::packed::CellInput> = vec![CellInput::new_builder()
            .previous_output(last_checkpoint_cell.out_point.into())
            .build()];

        let new_checkpoint_data: CheckpointCellData = self.new_checkpoint.clone().into();
        let outputs_data = vec![new_checkpoint_data
            .as_builder()
            .metadata_type_id(last_checkpoint_data.metadata_type_id()) // metdata type script hash
            .build()
            .as_bytes()];

        let outputs = vec![CellOutput::new_builder()
            .lock(always_success_lock(&self.ckb.network_type))
            .type_(Some(checkpoint_type).pack())
            .build_exact_capacity(Capacity::bytes(outputs_data[0].len())?)?];

        let cell_deps = vec![
            omni_lock_dep(&self.ckb.network_type),
            secp256k1_lock_dep(&self.ckb.network_type),
            xudt_type_dep(&self.ckb.network_type),
            always_success_lock_dep(&self.ckb.network_type),
            checkpoint_type_dep(&self.ckb.network_type),
            metadata_cell_dep(
                &self.ckb.client,
                &self.ckb.network_type,
                &self.type_ids.metadata_type_id, // metadata type script args
            )
            .await?,
        ];

        let witnesses = vec![
            Bytes::default(), // todo
            omni_eth_witness_placeholder().as_bytes(),
        ];

        let tx = TransactionBuilder::default()
            .inputs(inputs)
            .outputs(outputs)
            .outputs_data(outputs_data.pack())
            .cell_deps(cell_deps)
            .witnesses(witnesses.pack())
            .build();

        let kick_lock = omni_eth_lock(
            &self.ckb.network_type,
            &omni_eth_address(self.kicker_key.clone())?,
        );

        let tx = balance_tx(&self.ckb.client, kick_lock.clone(), tx).await?;

        let signer = omni_eth_signer(self.kicker_key.clone())?;
        let tx_view = signer.sign_tx(&tx, &ScriptGroup {
            script:         kick_lock,
            group_type:     ScriptGroupType::Lock,
            input_indices:  vec![1],
            output_indices: vec![],
        })?;

        Ok(tx_view)
    }
}

impl<C> CheckpointTxBuilder<C>
where
    C: CkbRpc,
{
    async fn check_occasion(&self, last_epoch: u64, last_period: u32) -> Result<(), CkbTxErr> {
        if (last_period as u64) == self.epoch_len - 1 {
            if self.new_checkpoint.period != 0 || self.new_checkpoint.epoch != last_epoch + 1 {
                Err(CkbTxErr::NotCheckpointOccasion {
                    current_epoch:   last_epoch,
                    current_period:  last_period,
                    recorded_epoch:  self.new_checkpoint.epoch,
                    recorded_period: self.new_checkpoint.period,
                })
            } else {
                Ok(())
            }
        } else if self.new_checkpoint.period != last_period + 1
            || self.new_checkpoint.epoch != last_epoch + 1
        {
            Err(CkbTxErr::NotCheckpointOccasion {
                current_epoch:   last_epoch,
                current_period:  last_period,
                recorded_epoch:  self.new_checkpoint.epoch,
                recorded_period: self.new_checkpoint.period,
            })
        } else {
            Ok(())
        }
    }
}
