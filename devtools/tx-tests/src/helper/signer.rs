use ckb_sdk::traits::{SecpCkbRawKeySigner, Signer};
use ckb_sdk::types::ScriptGroup;
use ckb_sdk::unlock::{ScriptSignError, ScriptSigner};
use ckb_types::{
    core::TransactionView,
    packed::{Bytes, BytesOpt, WitnessArgs},
    prelude::{Builder, Entity, Pack},
    H160, H256,
};

use common::types::axon_types::basic::Byte65;
use common::types::axon_types::stake::StakeAtWitness;
use tx_builder::ckb::helper::OmniEth;

pub struct EthSigner {
    pub signer:      Box<dyn Signer>,
    pub eth_address: H160,
    pub unlock_mode: UnlockMode,
}

#[allow(dead_code)]
pub enum UnlockMode {
    Stake,
    Delegate,
    Withdraw,
}

impl ScriptSigner for EthSigner {
    fn match_args(&self, _args: &[u8]) -> bool {
        true
    }

    fn sign_tx(
        &self,
        tx: &TransactionView,
        script_group: &ScriptGroup,
    ) -> Result<TransactionView, ScriptSignError> {
        let witness_idx = script_group.input_indices[0];
        let mut witnesses: Vec<Bytes> = tx.witnesses().into_iter().collect();
        while witnesses.len() <= witness_idx {
            witnesses.push(Default::default());
        }

        let message = tx.hash().as_bytes().to_vec();

        let signature = self
            .signer
            .sign(self.eth_address.as_ref(), message.as_ref(), true, tx)?;

        // Put signature into witness
        let witness_data = witnesses[witness_idx].raw_data();
        let mut current_witness: WitnessArgs = if witness_data.is_empty() {
            WitnessArgs::default()
        } else {
            WitnessArgs::from_slice(witness_data.as_ref())?
        };

        let lock = self.build_witness_lock(current_witness.lock(), signature)?;
        current_witness = current_witness.as_builder().lock(Some(lock).pack()).build();
        witnesses[witness_idx] = current_witness.as_bytes().pack();
        Ok(tx.as_advanced_builder().set_witnesses(witnesses).build())
    }
}

impl EthSigner {
    pub fn new(private_key: H256, unlock_mode: UnlockMode) -> Self {
        let key = secp256k1::SecretKey::from_slice(private_key.as_bytes()).unwrap();
        let signer = SecpCkbRawKeySigner::new_with_ethereum_secret_keys(vec![key]);
        Self {
            signer: Box::new(signer),
            eth_address: OmniEth::new(private_key).address().unwrap(),
            unlock_mode,
        }
    }

    fn build_witness_lock(
        &self,
        orig_lock: BytesOpt,
        signature: bytes::Bytes,
    ) -> Result<bytes::Bytes, ScriptSignError> {
        let lock_field = orig_lock.to_opt().map(|data| data.raw_data());
        match self.unlock_mode {
            UnlockMode::Stake => {
                let witness_lock = StakeAtWitness::from_slice(lock_field.as_ref().unwrap())?;
                Ok(witness_lock
                    .as_builder()
                    .eth_sig(Byte65::new_unchecked(signature))
                    .build()
                    .as_bytes())
            }
            UnlockMode::Delegate => unimplemented!(),
            UnlockMode::Withdraw => unimplemented!(),
        }
    }
}
