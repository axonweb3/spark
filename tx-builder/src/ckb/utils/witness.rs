use axon_types::delegate::DelegateAtWitness;
use axon_types::stake::StakeAtWitness;
use ckb_types::packed::WitnessArgs;
use ckb_types::prelude::{Builder, Pack};
use molecule::prelude::Entity;

// todo: eth sig placeholder
pub fn stake_witness_placeholder(mode: u8) -> WitnessArgs {
    let lock_field = StakeAtWitness::new_builder().mode(mode.into()).build();
    WitnessArgs::new_builder()
        .lock(Some(lock_field.as_bytes()).pack())
        .build()
}

// todo: eth sig placeholder
pub fn delegate_witness_placeholder(mode: u8) -> WitnessArgs {
    let lock_field = DelegateAtWitness::new_builder().mode(mode.into()).build();
    WitnessArgs::new_builder()
        .lock(Some(lock_field.as_bytes()).pack())
        .build()
}
