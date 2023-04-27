use async_trait::async_trait;
use std::collections::HashMap;

use sparse_merkle_tree::error::Error;

use crate::types::smt::{
    Address, Amount, Delegator, Epoch, Proof, ProposalCount, Root, Staker, Validator,
};

// High level business logic SMT APIs
#[async_trait]
pub trait StakeSmtStorage {
    async fn insert_stake(
        &self,
        epoch: Epoch,
        amounts: Vec<(Address, Amount)>,
    ) -> Result<(), Error>;

    async fn remove_stake(&self, epoch: Epoch, address: Address) -> Result<(), Error>;

    async fn remove_batch_stake(&self, epoch: Epoch, address: Vec<Address>) -> Result<(), Error>;

    async fn get_amount_stake(
        &self,
        epoch: Epoch,
        address: Address,
    ) -> Result<Option<Amount>, Error>;

    async fn get_sub_leaves_stake(&self, epoch: Epoch) -> Result<HashMap<Address, Amount>, Error>;

    async fn get_sub_root_stake(&self, epoch: Epoch) -> Result<Option<Root>, Error>;

    async fn get_sub_roots_stake(
        &self,
        epochs: Vec<Epoch>,
    ) -> Result<HashMap<Epoch, Option<Root>>, Error>;

    async fn get_top_root_stake(&self) -> Result<Root, Error>;

    async fn generate_sub_proof_stake(
        &self,
        epoch: Epoch,
        addresses: Vec<Address>,
    ) -> Result<Proof, Error>;

    async fn generate_top_proof_stake(&self, epochs: Vec<Epoch>) -> Result<Proof, Error>;
}

#[async_trait]
pub trait DelegateSmtStorage {
    async fn insert_delegate(
        &self,
        epoch: Epoch,
        delegators: HashMap<Staker, Vec<(Delegator, Amount)>>,
    ) -> Result<(), Error>;

    async fn remove_delegate(
        &self,
        epoch: Epoch,
        delegators: HashMap<Staker, Vec<Delegator>>,
    ) -> Result<(), Error>;

    async fn get_amount_delegate(
        &self,
        delegator: Delegator,
        staker: Staker,
        epoch: Epoch,
    ) -> Result<Option<Amount>, Error>;

    async fn get_sub_leaves_delegate(
        &self,
        staker: Staker,
        epoch: Epoch,
    ) -> Result<HashMap<Delegator, Amount>, Error>;

    async fn get_sub_root_delegate(
        &self,
        staker: Staker,
        epoch: Epoch,
    ) -> Result<Option<Root>, Error>;

    async fn get_sub_roots_delegate(
        &self,
        staker: Staker,
        epochs: Vec<Epoch>,
    ) -> Result<HashMap<Epoch, Option<Root>>, Error>;

    async fn get_top_root_delegate(&self, staker: Staker) -> Result<Root, Error>;

    async fn get_top_roots_delegate(
        &self,
        stakers: Vec<Staker>,
    ) -> Result<HashMap<Staker, Root>, Error>;

    async fn generate_sub_proof_delegate(
        &self,
        staker: Staker,
        epoch: Epoch,
        delegators: Vec<Delegator>,
    ) -> Result<Proof, Error>;

    async fn generate_top_proof_delegate(
        &self,
        staker: Staker,
        epochs: Vec<Epoch>,
    ) -> Result<Proof, Error>;
}

#[async_trait]
pub trait RewardSmtStorage {
    async fn insert_reward(&self, address: Address, epoch: Epoch) -> Result<(), Error>;

    async fn get_root_reward(&self) -> Result<Root, Error>;

    async fn get_epoch_reward(&self, address: Address) -> Result<Option<Epoch>, Error>;

    async fn generate_proof_reward(&self, addresses: Vec<Address>) -> Result<Proof, Error>;
}

#[async_trait]
pub trait ProposalSmtStorage {
    async fn insert_proposal(
        &self,
        epoch: Epoch,
        proposals: Vec<(Validator, ProposalCount)>,
    ) -> Result<(), Error>;

    async fn get_count_proposal(
        &self,
        epoch: Epoch,
    ) -> Result<HashMap<Validator, ProposalCount>, Error>;

    async fn get_sub_root_proposal(&self, epoch: Epoch) -> Result<Option<Root>, Error>;

    async fn get_sub_roots_proposal(
        &self,
        epochs: Vec<Epoch>,
    ) -> Result<HashMap<Epoch, Option<Root>>, Error>;

    async fn get_top_root_proposal(&self) -> Result<Root, Error>;

    async fn generate_sub_proof_proposal(
        &self,
        epoch: Epoch,
        validators: Vec<Validator>,
    ) -> Result<Proof, Error>;

    async fn generate_top_proof_proposal(&self, epochs: Vec<Epoch>) -> Result<Proof, Error>;
}
