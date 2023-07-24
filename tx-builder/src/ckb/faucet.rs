use anyhow::Result;
use ckb_sdk::{ScriptGroup, ScriptGroupType};
use ckb_types::{
    bytes::Bytes,
    core::{Capacity, TransactionBuilder, TransactionView},
    packed::CellOutput,
    prelude::{Entity, Pack},
};

use common::traits::ckb_rpc_client::CkbRpc;
use common::types::tx_builder::*;

use crate::ckb::helper::{OmniEth, Secp256k1, Tx};

use super::helper::sighash::Sighash;

pub struct FaucetTxBuilder<'a, C: CkbRpc> {
    ckb:        &'a C,
    seeder_key: PrivateKey,
    ckb_bytes:  u128,
}

impl<'a, C: CkbRpc> FaucetTxBuilder<'a, C> {
    pub fn new(ckb: &'a C, seeder_key: PrivateKey, ckb_bytes: u128) -> Self {
        Self {
            ckb,
            seeder_key,
            ckb_bytes,
        }
    }

    pub async fn build_tx(self) -> Result<TransactionView> {
        let omni_eth = OmniEth::new(self.seeder_key.clone());
        let seeder_omni_lock = OmniEth::lock(&omni_eth.address()?);

        let outputs = vec![
            // omni eth lock cell
            CellOutput::new_builder()
                .lock(seeder_omni_lock)
                .build_exact_capacity(Capacity::bytes(self.ckb_bytes as usize)?)?,
        ];

        let outputs_data = vec![Bytes::default()];

        let cell_deps = vec![Secp256k1::lock_dep()];

        let witnesses = vec![
            Sighash::witness_placeholder().as_bytes(), // capacity provider lock
        ];

        let tx = TransactionBuilder::default()
            .inputs(vec![])
            .outputs(outputs)
            .outputs_data(outputs_data.pack())
            .cell_deps(cell_deps)
            .witnesses(witnesses.pack())
            .build();

        let sig_hash = Sighash::new(self.seeder_key.clone());
        let sig_lock = sig_hash.lock()?;

        let mut tx = Tx::new(self.ckb, tx);
        tx.balance(sig_lock.clone()).await?;

        tx.sign(&sig_hash.signer()?, &ScriptGroup {
            script:         sig_lock.clone(),
            group_type:     ScriptGroupType::Lock,
            input_indices:  vec![0],
            output_indices: vec![],
        })?;

        Ok(tx.inner())
    }
}
