use anyhow::Result;
use async_trait::async_trait;
use axon_types::checkpoint::CheckpointCellData;
use ckb_types::{
    core::{Capacity, TransactionBuilder, TransactionView},
    packed::{CellOutput, Script},
    prelude::{Entity, Pack},
};

use common::traits::tx_builder::ICheckpointTxBuilder;
use common::types::tx_builder::{Checkpoint, PrivateKey};

pub struct CheckpointTxBuilder {
    _kicker:    PrivateKey,
    checkpoint: Checkpoint,
}

#[async_trait]
impl ICheckpointTxBuilder for CheckpointTxBuilder {
    fn new(_kicker: PrivateKey, checkpoint: Checkpoint) -> Self {
        Self {
            _kicker,
            checkpoint,
        }
    }

    async fn build_tx(&self) -> Result<TransactionView> {
        // todo: get checkpoint cell
        let inputs = vec![];

        let checkpoint: CheckpointCellData = (&self.checkpoint).into();
        let outputs_data = vec![checkpoint.as_bytes()];

        // todo: fill lock, type
        let fake_lock = Script::default();
        let fake_type = Script::default();
        let outputs = vec![
            // checkpoint cell
            CellOutput::new_builder()
                .lock(fake_lock)
                .type_(Some(fake_type).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[0].len())?)?,
        ];

        // todo
        let cell_deps = vec![];

        // todo: balance tx, fill placeholder witnesses,
        let tx = TransactionBuilder::default()
            .inputs(inputs)
            .outputs(outputs)
            .outputs_data(outputs_data.pack())
            .cell_deps(cell_deps)
            .build();

        // todo: sign tx

        Ok(tx)
    }
}
