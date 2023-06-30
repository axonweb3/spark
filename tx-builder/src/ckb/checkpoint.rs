use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use ckb_sdk::{unlock::ScriptSigner, ScriptGroup, ScriptGroupType};
use ckb_types::{
    core::{Capacity, TransactionBuilder, TransactionView},
    packed::{CellInput, CellOutput, WitnessArgs},
    prelude::{Entity, Pack},
};
use common::utils::convert::{to_u32, to_u64};
use common::{
    traits::{ckb_rpc_client::CkbRpc, tx_builder::ICheckpointTxBuilder},
    types::axon_types::basic::Bytes as ABytes,
    types::axon_types::checkpoint::{CheckpointCellData, CheckpointWitness},
    types::tx_builder::{Checkpoint, CheckpointProof, CheckpointTypeIds, PrivateKey},
};
use molecule::prelude::Builder;

use crate::ckb::{
    define::error::CkbTxErr,
    helper::{
        AlwaysSuccess, Checkpoint as HCheckpoint, Metadata as HMetadata, OmniEth, Secp256k1, Tx,
        Xudt,
    },
};

pub struct CheckpointTxBuilder<'a, C>
where
    C: CkbRpc,
{
    ckb:            &'a C,
    kicker_key:     PrivateKey,
    type_ids:       CheckpointTypeIds,
    epoch_len:      u64,
    new_checkpoint: Checkpoint,
    proof:          CheckpointProof,
}

#[async_trait]
impl<'a, C> ICheckpointTxBuilder<'a, C> for CheckpointTxBuilder<'a, C>
where
    C: CkbRpc,
{
    async fn new(
        ckb: &'a C,
        kicker_key: PrivateKey,
        type_ids: CheckpointTypeIds,
        epoch_len: u64,
        new_checkpoint: Checkpoint,
        proof: CheckpointProof,
    ) -> Self {
        Self {
            kicker_key,
            ckb,
            type_ids,
            epoch_len,
            new_checkpoint,
            proof,
        }
    }

    async fn build_tx(&self) -> Result<TransactionView> {
        let checkpoint_type = HCheckpoint::type_(&self.type_ids.checkpoint_type_id);

        let last_checkpoint_cell = HCheckpoint::get_cell(self.ckb, checkpoint_type.clone()).await?;

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
            .lock(AlwaysSuccess::lock())
            .type_(Some(checkpoint_type).pack())
            .build_exact_capacity(Capacity::bytes(outputs_data[0].len())?)?];

        let cell_deps = vec![
            OmniEth::lock_dep(),
            Secp256k1::lock_dep(),
            Xudt::type_dep(),
            AlwaysSuccess::lock_dep(),
            HCheckpoint::type_dep(),
            HMetadata::cell_dep(
                self.ckb,
                &self.type_ids.metadata_type_id, // metadata type script args
            )
            .await?,
        ];

        let witnesses = vec![
            WitnessArgs::new_builder()
                .input_type(
                    Some(
                        CheckpointWitness::new_builder()
                            .proof(ABytes::new_unchecked(self.proof.proof.bytes()))
                            .proposal(ABytes::new_unchecked(Bytes::from(
                                self.proof.proposal.hash().as_bytes().to_owned(),
                            )))
                            .build()
                            .as_bytes(),
                    )
                    .pack(),
                )
                .build()
                .as_bytes(),
            OmniEth::witness_placeholder().as_bytes(),
        ];

        let tx = TransactionBuilder::default()
            .inputs(inputs)
            .outputs(outputs)
            .outputs_data(outputs_data.pack())
            .cell_deps(cell_deps)
            .witnesses(witnesses.pack())
            .build();

        let omni_eth = OmniEth::new(self.kicker_key.clone());
        let kick_lock = OmniEth::lock(&omni_eth.address()?);

        let tx = Tx::new(self.ckb, tx).balance(kick_lock.clone()).await?;

        let signer = omni_eth.signer()?;
        let tx_view = signer.sign_tx(&tx, &ScriptGroup {
            script:         kick_lock,
            group_type:     ScriptGroupType::Lock,
            input_indices:  vec![1],
            output_indices: vec![],
        })?;

        Ok(tx_view)
    }
}

impl<'a, C> CheckpointTxBuilder<'a, C>
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
            || self.new_checkpoint.epoch != last_epoch
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
