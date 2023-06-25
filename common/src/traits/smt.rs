use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;

use crate::types::smt::{
    Address, Amount, Delegator, Epoch, Proof, ProposalCount, Root, Staker, UserAmount, Validator,
};

/// High level business logic SMT APIs for staker, delegator, proposal and
/// reward
#[async_trait]
pub trait StakeSmtStorage: Send + Sync {
    async fn new_epoch(&self, epoch: Epoch) -> Result<()>;

    async fn insert(&self, epoch: Epoch, stakers: Vec<UserAmount>) -> Result<()>;

    async fn remove(&self, epoch: Epoch, staker: Vec<Staker>) -> Result<()>;

    async fn get_amount(&self, epoch: Epoch, staker: Staker) -> Result<Option<Amount>>;

    async fn get_sub_leaves(&self, epoch: Epoch) -> Result<HashMap<Staker, Amount>>;

    async fn get_sub_root(&self, epoch: Epoch) -> Result<Option<Root>>;

    async fn get_sub_roots(&self, epochs: Vec<Epoch>) -> Result<HashMap<Epoch, Option<Root>>>;

    async fn get_top_root(&self) -> Result<Root>;

    async fn generate_sub_proof(&self, epoch: Epoch, stakers: Vec<Staker>) -> Result<Proof>;

    async fn generate_top_proof(&self, epochs: Vec<Epoch>) -> Result<Proof>;
}

#[async_trait]
pub trait DelegateSmtStorage: Send + Sync {
    async fn new_epoch(&self, epoch: Epoch) -> Result<()>;

    async fn insert(&self, epoch: Epoch, staker: Staker, delegators: Vec<UserAmount>)
        -> Result<()>;

    async fn remove(&self, epoch: Epoch, delegators: Vec<(Staker, Delegator)>) -> Result<()>;

    async fn get_amount(
        &self,
        epoch: Epoch,
        staker: Staker,
        delegator: Delegator,
    ) -> Result<Option<Amount>>;

    async fn get_sub_leaves(
        &self,
        epoch: Epoch,
        staker: Staker,
    ) -> Result<HashMap<Delegator, Amount>>;

    async fn get_sub_root(&self, epoch: Epoch, staker: Staker) -> Result<Option<Root>>;

    async fn get_sub_roots(
        &self,
        epochs: Vec<Epoch>,
        staker: Staker,
    ) -> Result<HashMap<Epoch, Option<Root>>>;

    async fn get_top_root(&self, staker: Staker) -> Result<Root>;

    async fn get_top_roots(&self, stakers: Vec<Staker>) -> Result<HashMap<Staker, Root>>;

    async fn generate_sub_proof(
        &self,
        staker: Staker,
        epoch: Epoch,
        delegators: Vec<Delegator>,
    ) -> Result<Proof>;

    async fn generate_top_proof(&self, epochs: Vec<Epoch>, staker: Staker) -> Result<Proof>;
}

#[async_trait]
pub trait RewardSmtStorage: Send + Sync {
    async fn insert(&self, epoch: Epoch, address: Address) -> Result<()>;

    async fn get_root(&self) -> Result<Root>;

    async fn get_epoch(&self, address: Address) -> Result<Option<Epoch>>;

    async fn generate_proof(&self, addresses: Vec<Address>) -> Result<Proof>;
}

#[async_trait]
pub trait ProposalSmtStorage: Send + Sync {
    async fn insert(&self, epoch: Epoch, proposals: Vec<(Validator, ProposalCount)>) -> Result<()>;

    async fn get_count(&self, epoch: Epoch, validator: Validator) -> Result<Option<ProposalCount>>;

    async fn get_sub_leaves(&self, epoch: Epoch) -> Result<HashMap<Validator, ProposalCount>>;

    async fn get_sub_root(&self, epoch: Epoch) -> Result<Option<Root>>;

    async fn get_sub_roots(&self, epochs: Vec<Epoch>) -> Result<HashMap<Epoch, Option<Root>>>;

    async fn get_top_root(&self) -> Result<Root>;

    async fn generate_sub_proof(&self, epoch: Epoch, validators: Vec<Validator>) -> Result<Proof>;

    async fn generate_top_proof(&self, epochs: Vec<Epoch>) -> Result<Proof>;
}
