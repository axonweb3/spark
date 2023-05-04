use anyhow::Result;
use async_trait::async_trait;
use axon_types::{basic::Byte32, checkpoint::CheckpointCellData};
use ckb_types::{
    bytes::Bytes,
    core::{Capacity, TransactionBuilder, TransactionView},
    packed::{CellOutput, Script},
    prelude::{Entity, Pack},
};

use common::traits::tx_builder::IInitTxBuilder;
use common::types::tx_builder::*;

use crate::ckb::utils::cell_data::*;

pub struct InitTxBuilder {
    _kicker:    PrivateKey,
    checkpoint: Checkpoint,
}

#[async_trait]
impl IInitTxBuilder for InitTxBuilder {
    fn new(_kicker: PrivateKey, checkpoint: Checkpoint) -> Self {
        Self {
            _kicker,
            checkpoint,
        }
    }

    async fn build_tx(&self) -> Result<TransactionView> {
        // todo: get ckb cell
        let inputs = vec![];

        // todo
        let outputs_data = self.build_data();

        // todo: fill lock, type
        let fake_lock = Script::default();
        let fake_type = Script::default();
        let outputs = vec![
            // selection cell
            CellOutput::new_builder()
                .lock(fake_lock.clone())
                .type_(Some(fake_type.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[0].len())?)?,
            // issue cell
            CellOutput::new_builder()
                .lock(fake_lock.clone())
                .type_(Some(fake_type.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[1].len())?)?,
            // checkpoint cell
            CellOutput::new_builder()
                .lock(fake_lock.clone())
                .type_(Some(fake_type.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[2].len())?)?,
            // metadata cell
            CellOutput::new_builder()
                .lock(fake_lock.clone())
                .type_(Some(fake_type.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[3].len())?)?,
            // stake smt cell
            CellOutput::new_builder()
                .lock(fake_lock.clone())
                .type_(Some(fake_type.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[4].len())?)?,
            // delegate smt cell
            CellOutput::new_builder()
                .lock(fake_lock.clone())
                .type_(Some(fake_type.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[5].len())?)?,
            // reward smt cell
            CellOutput::new_builder()
                .lock(fake_lock.clone())
                .type_(Some(fake_type.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[6].len())?)?,
            // AT cell
            CellOutput::new_builder()
                .lock(fake_lock.clone())
                .type_(Some(fake_type.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[8].len())?)?,
            // CKB cell
            CellOutput::new_builder()
                .lock(fake_lock)
                .type_(Some(fake_type).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[8].len())?)?,
        ];

        // todo
        let cell_deps = vec![
            // TypeID Type Deploy Cell
            // xUDT Type Deploy Cell
            // Selection Lock Deploy Cell
            // Omni Lock Group Cell
            // Secp Lock Group Cell
        ];

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

impl InitTxBuilder {
    fn build_data(&self) -> Vec<Bytes> {
        let checkpoint: CheckpointCellData = (&self.checkpoint).into();

        let stake_smt_root = Byte32::default(); // todo: create stake smt

        let delegate_smt_roots = vec![(Staker::default(), Byte32::default())]; // todo: create stake smt

        let reward_smt_root = Byte32::default(); // todo: create reward smt

        let proposal_smt_root = Byte32::default(); // todo: create proposal smt

        vec![
            // selection cell data
            Bytes::new(),
            // issue cell data
            Bytes::new(),
            // checkpoint cell data
            checkpoint.as_bytes(),
            // metadata cell data
            // stake smt cell data
            stake_smt_cell_data(stake_smt_root).as_bytes(),
            // delegate smt cell data
            delegate_smt_cell_data(delegate_smt_roots).as_bytes(),
            // reward smt cell data
            reward_smt_cell_data(reward_smt_root).as_bytes(),
            // proposal smt cell data
            proposal_smt_root.as_bytes(),
            // AT cell data
        ]
    }
}
