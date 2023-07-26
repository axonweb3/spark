use anyhow::Result;
use ckb_jsonrpc_types::{OutputsValidator, Status};
use ckb_sdk::types::ScriptGroup;
use ckb_sdk::unlock::ScriptSigner;
use ckb_types::{
    core::{Capacity, TransactionView},
    packed::{Byte32, Bytes, CellInput, CellOutput, Script},
    prelude::*,
    H256,
};
use linked_hash_map::LinkedHashMap;

use common::traits::ckb_rpc_client::CkbRpc;
use common::types::ckb_rpc_client::{ScriptType, SearchKey};
use common::types::TransactionWithStatusResponse;

use crate::ckb::define::constants::FEE_RATE;
use crate::ckb::define::error::CkbTxErr;
use crate::ckb::helper::ckb::cell_collector::{get_live_cell, get_live_cells};

const KB: u64 = 1000;

pub struct Tx<'a, C: CkbRpc> {
    rpc:     &'a C,
    tx:      TransactionView,
    tx_hash: H256,
}

pub struct ScriptGroups {
    pub lock_groups: LinkedHashMap<Byte32, ScriptGroup>,
    pub type_groups: LinkedHashMap<Byte32, ScriptGroup>,
}

impl<'a, C: CkbRpc> Tx<'a, C> {
    pub fn new(rpc: &'a C, tx: TransactionView) -> Self {
        Self {
            rpc,
            tx,
            tx_hash: H256::default(),
        }
    }

    pub fn inner(self) -> TransactionView {
        self.tx
    }

    pub fn inner_clone(&self) -> TransactionView {
        self.tx.clone()
    }

    pub fn inner_ref(&self) -> &TransactionView {
        &self.tx
    }

    pub fn set_tx(&mut self, tx: TransactionView) {
        self.tx = tx;
    }

    /// There is no pure CKB cell in the input and output of the transaction.
    /// Collect CKB cells and add them to the input of the transaction.
    /// Add a CKB change cell to the output of the transaction.
    pub async fn balance(&mut self, capacity_provider: Script) -> Result<()> {
        let outputs_capacity = self.add_ckb_to_outputs(capacity_provider.clone())?;

        let inputs_capacity = self
            .add_ckb_to_intputs(capacity_provider.clone(), outputs_capacity)
            .await?;

        self.change_ckb(inputs_capacity, outputs_capacity)?;

        Ok(())
    }

    pub fn sign(&mut self, signer: &impl ScriptSigner, script_group: &ScriptGroup) -> Result<()> {
        self.tx = signer.sign_tx(&self.tx, script_group)?;
        Ok(())
    }

    pub async fn send(&mut self) -> Result<String> {
        let outputs_validator = Some(OutputsValidator::Passthrough);
        self.tx_hash = self
            .rpc
            .send_transaction(&(self.tx.data().into()), outputs_validator)
            .await?;
        Ok(self.tx_hash.to_string())
    }

    pub async fn query_status(&self) -> Result<Option<TransactionWithStatusResponse>> {
        self.rpc.get_transaction(self.tx_hash.clone()).await
    }

    pub async fn wait_until_committed(&self, interval_ms: u64, max_try: u64) -> Result<()> {
        let mut status = Status::Proposed;
        let mut try_count = 0;

        while status != Status::Committed {
            if let Some(tx_with_status) = self.rpc.get_transaction(self.tx_hash.clone()).await? {
                status = tx_with_status.tx_status.status;
            }

            try_count += 1;
            if try_count >= max_try {
                break;
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(interval_ms)).await;
        }

        Ok(())
    }

    pub async fn gen_script_group(&self) -> Result<ScriptGroups> {
        #[allow(clippy::mutable_key_type)]
        let mut lock_groups: LinkedHashMap<Byte32, ScriptGroup> = LinkedHashMap::default();
        #[allow(clippy::mutable_key_type)]
        let mut type_groups: LinkedHashMap<Byte32, ScriptGroup> = LinkedHashMap::default();

        for (i, input) in self.tx.inputs().into_iter().enumerate() {
            let output = self
                .rpc
                .get_live_cell(input.previous_output().into(), true)
                .await?;
            let output: CellOutput = output.cell.unwrap().output.into();

            let lock_group_entry = lock_groups
                .entry(output.calc_lock_hash())
                .or_insert_with(|| ScriptGroup::from_lock_script(&output.lock()));
            lock_group_entry.input_indices.push(i);

            if let Some(t) = &output.type_().to_opt() {
                let type_group_entry = type_groups
                    .entry(t.calc_script_hash())
                    .or_insert_with(|| ScriptGroup::from_type_script(t));
                type_group_entry.input_indices.push(i);
            }
        }

        for (i, output) in self.tx.outputs().into_iter().enumerate() {
            if let Some(t) = &output.type_().to_opt() {
                let type_group_entry = type_groups
                    .entry(t.calc_script_hash())
                    .or_insert_with(|| ScriptGroup::from_type_script(t));
                type_group_entry.output_indices.push(i);
            }
        }

        Ok(ScriptGroups {
            lock_groups,
            type_groups,
        })
    }

    fn add_ckb_to_outputs(&mut self, capacity_provider: Script) -> Result<u64> {
        let mut outputs = self.tx.outputs().into_iter().collect::<Vec<_>>();
        let mut outputs_data = self.tx.outputs_data().into_iter().collect::<Vec<_>>();

        outputs.push(
            CellOutput::new_builder()
                .lock(capacity_provider)
                .build_exact_capacity(Capacity::zero())?,
        );

        outputs_data.push(Bytes::default());

        let outputs_capacity = Self::calc_outputs_capacity(&outputs);

        self.tx = self
            .tx
            .as_advanced_builder()
            .set_outputs(outputs)
            .set_outputs_data(outputs_data)
            .build();

        Ok(outputs_capacity)
    }

    async fn add_ckb_to_intputs(
        &mut self,
        capacity_provider: Script,
        outputs_capacity: u64,
    ) -> Result<u64> {
        let mut inputs = self.tx.inputs().into_iter().collect::<Vec<_>>();

        let (mut extra_inputs, inputs_capacity) = get_live_cells(
            self.rpc,
            SearchKey {
                script:               capacity_provider.into(),
                script_type:          ScriptType::Lock,
                filter:               None,
                script_search_mode:   None,
                with_data:            Some(false),
                group_by_transaction: None,
            },
            self.calc_inputs_capacity(&inputs).await?,
            outputs_capacity + Capacity::bytes(1)?.as_u64(),
        )
        .await?;

        inputs.append(&mut extra_inputs);

        self.tx = self.tx.as_advanced_builder().set_inputs(inputs).build();

        Ok(inputs_capacity)
    }

    fn change_ckb(&mut self, inputs_capacity: u64, outputs_capacity: u64) -> Result<()> {
        let tx_size = self.tx.data().as_reader().serialized_size_in_block();
        let needed_capacity = outputs_capacity + Self::fee(tx_size).as_u64();

        if inputs_capacity < needed_capacity {
            return Err(CkbTxErr::InsufficientCapacity {
                inputs_capacity,
                outputs_capacity: needed_capacity,
            }
            .into());
        }

        let change = inputs_capacity - needed_capacity;

        let mut outputs = self.tx.outputs().into_iter().collect::<Vec<_>>();
        let idx = outputs.len() - 1;
        let old_capacity: u64 = outputs[idx].capacity().unpack();
        let new_capacity = old_capacity
            .checked_add(change)
            .expect("change cell capacity add overflow");
        outputs[idx] = self
            .tx
            .output(idx)
            .expect("last output")
            .as_builder()
            .capacity(new_capacity.pack())
            .build();

        self.tx = self.tx.as_advanced_builder().set_outputs(outputs).build();
        Ok(())
    }

    fn fee(tx_size: usize) -> Capacity {
        let fee = FEE_RATE.saturating_mul(tx_size as u64) / KB;
        Capacity::shannons(fee)
    }

    async fn calc_inputs_capacity(&self, inputs: &[CellInput]) -> Result<u64> {
        let mut inputs_capacity: u64 = 0;
        for input in inputs.iter() {
            let cell = get_live_cell(self.rpc, input.previous_output(), false).await?;
            let output = CellOutput::from(cell.output);
            let input_capacity: u64 = output.capacity().unpack();
            inputs_capacity += input_capacity;
        }
        Ok(inputs_capacity)
    }

    fn calc_outputs_capacity(outputs: &[CellOutput]) -> u64 {
        outputs
            .iter()
            .map(|output| output.capacity().unpack())
            .collect::<Vec<u64>>()
            .iter()
            .sum::<u64>()
    }
}
