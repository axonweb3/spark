use std::collections::HashMap;

use sparse_merkle_tree::error::Error;

use crate::types::smt::{Address, Amount, Delegator, Epoch, Proof, Root, Staker};

// High level business logic SMT APIs
pub trait StakeSmtStorage {
    fn insert(&self, epoch: Epoch, staker_infos: Vec<(Staker, Amount)>) -> Result<(), Error>;

    fn remove(&self, epoch: Epoch, staker: Staker) -> Result<(), Error>;

    fn remove_batch(&self, epoch: Epoch, stakers: Vec<Staker>) -> Result<(), Error>;

    fn get_amount(&self, epoch: Epoch, staker: Staker) -> Result<Option<Amount>, Error>;

    fn get_sub_leaves(&self, epoch: Epoch) -> Result<HashMap<Staker, Amount>, Error>;

    fn get_sub_root(&self, epoch: Epoch) -> Result<Option<Root>, Error>;

    fn get_sub_roots(&self, epochs: Vec<Epoch>) -> Result<HashMap<Epoch, Option<Root>>, Error>;

    fn get_top_root(&self) -> Result<Option<Root>, Error>;

    fn generate_sub_proof(&self, epoch: Epoch, stakers: Vec<Staker>) -> Result<Proof, Error>;

    fn generate_top_proof(&self, epochs: Vec<Epoch>) -> Result<Proof, Error>;
}

pub trait DelegateSmtStorage {
    fn insert(&self, epoch: Epoch, delegator_info: (Staker, Delegator, Amount)) -> Result<(), Error>;

    fn insert_batch(
        &self,
        epoch: Epoch,
        delegator_infos: Vec<(Staker, Delegator, Amount)>,
    ) -> Result<(), Error>;

    fn remove(&self, epoch: Epoch, delegator_info: (Staker, Delegator)) -> Result<(), Error>;

    fn remove_batch(
        &self,
        epoch: Epoch,
        delegator_infos: Vec<(Staker, Delegator)>,
    ) -> Result<(), Error>;

    fn get_amount(
        &self,
        delegator: &Delegator,
        staker: &Staker,
        epoch: Epoch,
    ) -> Result<Amount, Error>;

    fn get_sub_leaves(&self, staker: &Staker, epoch: Epoch) -> Result<HashMap<Delegator, Amount>, Error>;

    fn get_sub_root(&self, staker: &Staker, epoch: Epoch) -> Result<Root, Error>;

    fn get_sub_roots(
        &self,
        staker: &Staker,
        epochs: Vec<Epoch>,
    ) -> Result<HashMap<Epoch, Root>, Error>;

    fn get_top_root(&self, staker: &Staker) -> Result<Root, Error>;

    fn get_top_roots(&self, stakers: Vec<Staker>) -> Result<HashMap<Staker, Root>, Error>;

    fn generate_sub_proof(
        &self,
        staker: &Staker,
        epoch: Epoch,
        delegators: Vec<Delegator>,
    ) -> Result<Proof, Error>;

    fn generate_top_proof(&self, staker: &Staker, epochs: Vec<Epoch>) -> Result<Proof, Error>;
}

pub trait RewardSmtStorage {
    fn insert(&self, address: &Address, epoch: Epoch) -> Result<(), Error>;

    fn get_root(&self) -> Result<Root, Error>;

    fn generate_proof(&self, addresses: Vec<Address>) -> Result<Proof, Error>;
}
