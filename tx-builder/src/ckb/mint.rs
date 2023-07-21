use anyhow::Result;
use async_trait::async_trait;
use ckb_sdk::{ScriptGroup, ScriptGroupType};
use ckb_types::{
    bytes::Bytes,
    core::{Capacity, TransactionBuilder, TransactionView},
    packed::{CellInput, CellOutput, Script},
    prelude::{Entity, Pack},
    H256,
};
use molecule::prelude::Builder;

use common::traits::ckb_rpc_client::CkbRpc;
use common::traits::tx_builder::IMintTxBuilder;
use common::types::axon_types::issue::IssueCellData;
use common::types::ckb_rpc_client::Cell;
use common::types::tx_builder::*;
use common::utils::convert::{to_u128, to_uint128};

use crate::ckb::define::error::CkbTxErr;
use crate::ckb::helper::{Issue, OmniEth, Secp256k1, Selection, Tx, Xudt};

pub struct MintTxBuilder<'a, C: CkbRpc> {
    ckb:               &'a C,
    seeder_key:        PrivateKey,
    stakers:           Vec<(StakerEthAddr, Amount)>,
    selection_type_id: H256,
    issue_type_id:     H256,
}

#[async_trait]
impl<'a, C: CkbRpc> IMintTxBuilder<'a, C> for MintTxBuilder<'a, C> {
    fn new(
        ckb: &'a C,
        seeder_key: PrivateKey,
        stakers: Vec<(StakerEthAddr, Amount)>,
        selection_type_id: H256,
        issue_type_id: H256,
    ) -> Self {
        Self {
            ckb,
            seeder_key,
            stakers,
            selection_type_id,
            issue_type_id,
        }
    }

    async fn build_tx(self) -> Result<TransactionView> {
        let omni_eth = OmniEth::new(self.seeder_key.clone());
        let seeder_lock = OmniEth::lock(&omni_eth.address()?);

        let selection_cell = Selection::get_cell(self.ckb, &self.selection_type_id).await?;
        let issue_cell = Issue::get_cell(self.ckb, &self.issue_type_id).await?;

        let inputs = vec![
            // selection cell
            CellInput::new_builder()
                .previous_output(selection_cell.out_point.clone().into())
                .build(),
            // issue cell
            CellInput::new_builder()
                .previous_output(issue_cell.out_point.clone().into())
                .build(),
        ];

        let (outputs, outputs_data) = self.fill_outputs(selection_cell, issue_cell)?;

        let cell_deps = vec![
            OmniEth::lock_dep(),
            Secp256k1::lock_dep(),
            Xudt::type_dep(),
            Selection::lock_dep(),
        ];

        let witnesses = vec![
            Bytes::default(),                          // selection lock & type
            OmniEth::witness_placeholder().as_bytes(), // issue lock
            OmniEth::witness_placeholder().as_bytes(), // capacity provider lock
        ];

        let tx = TransactionBuilder::default()
            .inputs(inputs)
            .outputs(outputs)
            .outputs_data(outputs_data.pack())
            .cell_deps(cell_deps)
            .witnesses(witnesses.pack())
            .build();

        let mut tx = Tx::new(self.ckb, tx);
        tx.balance(seeder_lock.clone()).await?;

        let signer = OmniEth::new(self.seeder_key.clone()).signer()?;
        tx.sign(&signer, &ScriptGroup {
            script:         seeder_lock.clone(),
            group_type:     ScriptGroupType::Lock,
            input_indices:  vec![1],
            output_indices: vec![],
        })?;
        tx.sign(&signer, &ScriptGroup {
            script:         seeder_lock.clone(),
            group_type:     ScriptGroupType::Lock,
            input_indices:  vec![2],
            output_indices: vec![],
        })?;

        Ok(tx.inner())
    }
}

impl<'a, C: CkbRpc> MintTxBuilder<'a, C> {
    fn fill_outputs(
        &self,
        selection_cell: Cell,
        issue_cell: Cell,
    ) -> Result<(Vec<CellOutput>, Vec<Bytes>)> {
        let mut outputs_data = vec![];
        let mut outputs = vec![];

        let selection_lock: Script = selection_cell.output.lock.clone().into();
        let xudt = Xudt::type_(&selection_lock.calc_script_hash());

        let issue_data = IssueCellData::new_unchecked(issue_cell.output_data.unwrap().into_bytes());

        let max_supply = to_u128(&issue_data.max_suppley());
        let current_supply = to_u128(&issue_data.current_supply());
        let mut total_mint = 0;

        // mint cells
        for (staker, amount) in self.stakers.iter() {
            outputs_data.push(amount.pack().as_bytes());
            outputs.push(
                CellOutput::new_builder()
                    .lock(OmniEth::lock(staker))
                    .type_(Some(xudt.clone()).pack())
                    .build_exact_capacity(Capacity::bytes(16)?)?,
            );

            total_mint += amount;
            if total_mint > max_supply {
                return Err(CkbTxErr::ExceedMaxSupply {
                    max_supply,
                    total_mint,
                }
                .into());
            }
        }

        // selection cell
        outputs_data.push(Bytes::default());
        outputs.push(
            CellOutput::new_builder()
                .lock(selection_cell.output.lock.into())
                .type_(Some(selection_cell.output.type_.unwrap().into()).pack())
                .build_exact_capacity(Capacity::zero())?,
        );

        // issue cell
        outputs_data.push(
            issue_data
                .as_builder()
                .current_supply(to_uint128(current_supply + total_mint))
                .build()
                .as_bytes(),
        );
        outputs.push(
            CellOutput::new_builder()
                .lock(issue_cell.output.lock.into())
                .type_(Some(issue_cell.output.type_.unwrap().into()).pack())
                .build_exact_capacity(Capacity::bytes(outputs_data.last().unwrap().len())?)?,
        );

        Ok((outputs, outputs_data))
    }
}
