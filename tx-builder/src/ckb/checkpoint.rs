use anyhow::Result;
use async_trait::async_trait;
use axon_types::checkpoint::CheckpointCellData;
use bytes::Bytes;
use ckb_sdk::{unlock::ScriptSigner, ScriptGroup, ScriptGroupType};
use ckb_types::{
    core::{Capacity, TransactionBuilder, TransactionView},
    packed::{CellInput, CellOutput, Script},
    prelude::{Entity, Pack},
};
use common::{
    traits::{ckb_rpc_client::CkbRpc, tx_builder::ICheckpointTxBuilder},
    types::{
        ckb_rpc_client::Cell,
        tx_builder::{CheckpointTypeIds, CkbNetwork},
    },
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
        cell_dep::{metadata_cell_dep, omni_lock_dep, secp256k1_lock_dep, xudt_type_dep},
        omni::{omni_eth_address, omni_eth_signer, omni_eth_witness_placeholder},
        script::{always_success_lock, checkpoint_type, omni_eth_lock},
        tx::balance_tx,
    },
};

pub struct CheckpointTxBuilder<C>
where
    C: CkbRpc,
{
    kicker_key:             PrivateKey,
    ckb:                    CkbNetwork<C>,
    type_ids:               CheckpointTypeIds,
    latest_checkpoint_info: Checkpoint,
    checkpoint_type_script: Script,
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
        latest_checkpoint_info: Checkpoint,
    ) -> Self {
        let checkpoint_type_script =
            checkpoint_type(&ckb.network_type, &type_ids.checkpoint_type_id);
        Self {
            kicker_key,
            ckb,
            type_ids,
            latest_checkpoint_info,
            checkpoint_type_script,
        }
    }

    async fn build_tx(&self) -> Result<TransactionView> {
        let last_checkpoint_cell =
            get_unique_cell(&self.ckb.client, self.checkpoint_type_script.clone()).await?;
        self.check_occasion(last_checkpoint_cell.clone()).await?;

        let checkpoint_lock = always_success_lock(&self.ckb.network_type);
        let kicker_addr = omni_eth_address(self.kicker_key.clone())?;
        let kick_token_lock = omni_eth_lock(&self.ckb.network_type, &kicker_addr);

        let new_checkpoint_cell_data: CheckpointCellData = (&self.latest_checkpoint_info).into();
        let outputs_data = vec![new_checkpoint_cell_data.as_bytes()];

        let inputs: Vec<ckb_types::packed::CellInput> = vec![CellInput::new_builder()
            .previous_output(last_checkpoint_cell.clone().out_point.into())
            .build()];

        let outputs = vec![CellOutput::new_builder()
            .lock(checkpoint_lock.clone())
            .type_(Some(self.checkpoint_type_script.clone()).pack())
            .build_exact_capacity(Capacity::bytes(outputs_data[0].len())?)?];

        let cell_deps = vec![
            omni_lock_dep(&self.ckb.network_type),
            secp256k1_lock_dep(&self.ckb.network_type),
            xudt_type_dep(&self.ckb.network_type),
            metadata_cell_dep(
                &self.ckb.client,
                &self.ckb.network_type,
                &self.type_ids.metadata_type_id,
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

        let tx = balance_tx(&self.ckb.client, kick_token_lock.clone(), tx).await?;

        let signer = omni_eth_signer(self.kicker_key.clone())?;

        let tx_view = signer.sign_tx(&tx, &ScriptGroup {
            script:         kick_token_lock,
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
    async fn check_occasion(&self, last_checkpoint_cell: Cell) -> Result<(), CkbTxErr> {
        let last_checkpoint_cell_data = CheckpointCellData::new_unchecked(
            last_checkpoint_cell.output_data.unwrap().into_bytes(),
        );
        let last_epoch = to_u64(&last_checkpoint_cell_data.epoch());
        let last_period = to_u32(&last_checkpoint_cell_data.period());
        match self.latest_checkpoint_info.epoch {
            latest_epoch if latest_epoch == last_epoch.saturating_add(1) => {
                match self.latest_checkpoint_info.period {
                    0 => Ok(()),
                    _ => Err(CkbTxErr::NotCheckpointOccasion {
                        current_epoch:   last_epoch,
                        current_period:  last_period,
                        recorded_epoch:  latest_epoch,
                        recorded_period: self.latest_checkpoint_info.period,
                    }),
                }
            }
            latest_epoch if latest_epoch == last_epoch => {
                match self.latest_checkpoint_info.period {
                    last_period if last_period == last_period.saturating_add(1) => Ok(()),
                    _ => Err(CkbTxErr::NotCheckpointOccasion {
                        current_epoch:   last_epoch,
                        current_period:  last_period,
                        recorded_epoch:  latest_epoch,
                        recorded_period: last_period,
                    }),
                }
            }
            _ => Err(CkbTxErr::NotCheckpointOccasion {
                current_epoch:   last_epoch,
                current_period:  last_period,
                recorded_epoch:  self.latest_checkpoint_info.epoch,
                recorded_period: self.latest_checkpoint_info.period,
            }),
        }
    }
}
