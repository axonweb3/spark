use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;

use crate::types::smt::{
    Address, Amount, Delegator, Epoch, Proof, ProposalCount, Root, Staker, UserAmount, Validator,
};

/// High level business logic SMT APIs for staker, delegator, proposal and
/// reward
#[async_trait]
pub trait StakeSmtStorage {
    async fn insert(&self, epoch: Epoch, amounts: Vec<UserAmount>) -> Result<()>;

    async fn remove(&self, epoch: Epoch, address: Address) -> Result<()>;

    async fn remove_batch(&self, epoch: Epoch, address: Vec<Address>) -> Result<()>;

    async fn get_amount(&self, epoch: Epoch, address: Address) -> Result<Option<Amount>>;

    async fn get_sub_leaves(&self, epoch: Epoch) -> Result<HashMap<Address, Amount>>;

    async fn get_sub_root(&self, epoch: Epoch) -> Result<Option<Root>>;

    async fn get_sub_roots(&self, epochs: Vec<Epoch>) -> Result<HashMap<Epoch, Option<Root>>>;

    async fn get_top_root(&self) -> Result<Root>;

    async fn generate_sub_proof(&self, epoch: Epoch, addresses: Vec<Address>) -> Result<Proof>;

    async fn generate_top_proof(&self, epochs: Vec<Epoch>) -> Result<Proof>;
}

#[async_trait]
pub trait DelegateSmtStorage {
    async fn insert(&self, epoch: Epoch, delegators: Vec<(Staker, UserAmount)>) -> Result<()>;

    async fn remove(&self, epoch: Epoch, delegators: Vec<(Staker, Delegator)>) -> Result<()>;

    async fn get_amount(
        &self,
        delegator: Delegator,
        staker: Staker,
        epoch: Epoch,
    ) -> Result<Option<Amount>>;

    async fn get_sub_leaves(
        &self,
        staker: Staker,
        epoch: Epoch,
    ) -> Result<HashMap<Delegator, Amount>>;

    async fn get_sub_root(&self, staker: Staker, epoch: Epoch) -> Result<Option<Root>>;

    async fn get_sub_roots(
        &self,
        staker: Staker,
        epochs: Vec<Epoch>,
    ) -> Result<HashMap<Epoch, Option<Root>>>;

    async fn get_top_root(&self, staker: Staker) -> Result<Root>;

    async fn get_top_roots(&self, stakers: Vec<Staker>) -> Result<HashMap<Staker, Root>>;

    async fn generate_sub_proof(
        &self,
        staker: Staker,
        epoch: Epoch,
        delegators: Vec<Delegator>,
    ) -> Result<Proof>;

    async fn generate_top_proof(&self, staker: Staker, epochs: Vec<Epoch>) -> Result<Proof>;
}

#[async_trait]
pub trait RewardSmtStorage {
    async fn insert(&self, address: Address, epoch: Epoch) -> Result<()>;

    async fn get_root(&self) -> Result<Root>;

    async fn get_epoch(&self, address: Address) -> Result<Option<Epoch>>;

    async fn generate_proof(&self, addresses: Vec<Address>) -> Result<Proof>;
}

#[async_trait]
pub trait ProposalSmtStorage {
    async fn insert(&self, epoch: Epoch, proposals: Vec<(Validator, ProposalCount)>) -> Result<()>;

    async fn get_count(&self, epoch: Epoch, validator: Validator) -> Result<Option<ProposalCount>>;

    async fn get_sub_leaves(&self, epoch: Epoch) -> Result<HashMap<Validator, ProposalCount>>;

    async fn get_sub_root(&self, epoch: Epoch) -> Result<Option<Root>>;

    async fn get_sub_roots(&self, epochs: Vec<Epoch>) -> Result<HashMap<Epoch, Option<Root>>>;

    async fn get_top_root(&self) -> Result<Root>;

    async fn generate_sub_proof(&self, epoch: Epoch, validators: Vec<Validator>) -> Result<Proof>;

    async fn generate_top_proof(&self, epochs: Vec<Epoch>) -> Result<Proof>;
}
