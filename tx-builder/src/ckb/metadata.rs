use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use ckb_types::{
    bytes::Bytes,
    core::{Capacity, TransactionBuilder, TransactionView},
    packed::{CellOutput, Script},
    prelude::{Entity, Pack},
};

use common::traits::tx_builder::IMetadataTxBuilder;
use common::types::tx_builder::*;

pub struct MetadataSmtTxBuilder {
    _kicker: PrivateKey,
    _quorum: u16,
}

#[async_trait]
impl IMetadataTxBuilder for MetadataSmtTxBuilder {
    fn new(_kicker: PrivateKey, _quorum: u16) -> Self {
        Self { _kicker, _quorum }
    }

    async fn build_tx(&self) -> Result<(TransactionView, NonTopStakers, NonTopDelegators)> {
        // todo: get metadata cell, stake smt cell, delegate smt cell
        let inputs = vec![];

        // todo
        let outputs_data = vec![Bytes::default()];

        // todo: fill lock, type
        let fake_lock = Script::default();
        let fake_type = Script::default();
        let outputs = vec![
            // metadata cell
            CellOutput::new_builder()
                .lock(fake_lock.clone())
                .type_(Some(fake_type.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[0].len())?)?,
            // stake smt cell
            CellOutput::new_builder()
                .lock(fake_lock.clone())
                .type_(Some(fake_type.clone()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[0].len())?)?,
            // delegate smt cell
            CellOutput::new_builder()
                .lock(fake_lock)
                .type_(Some(fake_type).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data[0].len())?)?,
        ];

        // todo: add removed stakers' stake AT cells to inputs and outputs and
        //       add withdraw AT cells to outputs

        // todo: add removed delegators' delegate AT cells to inputs and outputs and
        //       add withdraw AT cells to outputs

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

        Ok((tx, HashMap::default(), HashMap::default()))
    }
}
