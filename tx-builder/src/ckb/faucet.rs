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
    users:      Vec<(EthAddress, Amount)>,
}

impl<'a, C: CkbRpc> FaucetTxBuilder<'a, C> {
    pub fn new(ckb: &'a C, seeder_key: PrivateKey, stakers: Vec<(StakerEthAddr, Amount)>) -> Self {
        Self {
            ckb,
            seeder_key,
            users: stakers,
        }
    }

    pub async fn build_tx(self) -> Result<TransactionView> {
        let mut outputs = vec![];
        let mut outputs_data = vec![];

        // omni eth lock cells
        for (user, ckb_bytes) in self.users.into_iter() {
            outputs.push(
                CellOutput::new_builder()
                    .lock(OmniEth::lock(&user))
                    .build_exact_capacity(Capacity::bytes(ckb_bytes as usize)?)?,
            );
            outputs_data.push(Bytes::default());
        }

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
