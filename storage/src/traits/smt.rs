use std::{collections::HashMap, sync::Arc};

use sparse_merkle_tree::error::Error;

use rocksdb::DB;

use crate::{types::smt::{Address, Amount, Delegator, Epoch, Leaf, Proof, Root, Staker}, smt::Smt};

// Low level single SMT APIs
pub trait SmtStorage {
    fn insert(&mut self, key: Address, value: Amount) -> Result<(), Error>;

    fn get(&self, key: Address) -> Result<Option<Amount>, Error>;

    fn remove(&mut self, address: Address) -> Result<(), Error>;

    fn compute_root(&self, leaves: Vec<Leaf>) -> Result<Root, Error>;

    fn verify_root(&self, root_hash: Root, leaves: Vec<Leaf>) -> Result<bool, Error>;

    fn save_db(&self, db: DB) -> Result<(), Error>;

    fn root(&self) -> Result<Root, Error>;
}

// Mid level multiple SMTs APIs
// in memory, needs persistence storage
pub trait SmtMapManager {
    fn get_smt(&self, key: Address) -> Option<Arc<Smt>>;

    fn get_roots(&self, keys: Vec<Address>) -> Result<HashMap<Address, Root>, Error>;

    fn insert_smt(&self, key: Address, smt: Arc<Smt>) -> Result<(), Error>;

    fn remove_smt(&self, key: Address) -> Result<(), Error>;

    fn load(&self, keys: Vec<Address>) -> Result<Smt, Error>;
}

// High level business logic SMT APIs
pub trait StakeSmtManager {
    fn insert(&self, epoch: Epoch, staker_infos: (Staker, Amount)) -> Result<(), Error>;

    fn insert_batch(&self, epoch: Epoch, staker_infos: Vec<(Staker, Amount)>) -> Result<(), Error>;

    fn remove(&self, epoch: Epoch, staker: Staker) -> Result<(), Error>;

    fn remove_batch(&self, epoch: Epoch, stakers: Vec<Staker>) -> Result<(), Error>;

    fn get_amount(&self, staker: Staker, epoch: Epoch) -> Result<Amount, Error>;

    fn get_sub_leaves(&self, epoch: Epoch) -> Result<Vec<Leaf>, Error>;

    fn get_sub_root(&self, epoch: Epoch) -> Result<Root, Error>;

    fn get_sub_roots(&self, epochs: Vec<Epoch>) -> Result<HashMap<Epoch, Root>, Error>;

    fn get_top_root(&self) -> Result<Root, Error>;

    fn generate_sub_proof(&self, epoch: Epoch, stakers: Vec<Staker>) -> Result<Proof, Error>;

    fn generate_top_proof(&self, epochs: Vec<Epoch>) -> Result<Proof, Error>;
}

pub trait DelegateSmtManager {
    fn insert(
        &self,
        epoch: Epoch,
        delegator_info: (Staker, Delegator, Amount),
    ) -> Result<(), Error>;

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

    fn get_sub_leaves(&self, staker: &Staker, epoch: Epoch) -> Result<Vec<Leaf>, Error>;

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

pub trait RewardSmtManager {
    fn insert(&self, address: &Address, epoch: Epoch) -> Result<(), Error>;

    fn get_root(&self) -> Result<Root, Error>;

    fn generate_proof(&self, addresses: Vec<Address>) -> Result<Proof, Error>;
}

pub trait ExpiredRewardSmtManager {
    fn insert(&self, address: &Address, amount: Amount) -> Result<(), Error>;

    fn get_root(&self) -> Result<Root, Error>;

    fn generate_proof(&self, addresses: Vec<Address>) -> Result<Proof, Error>;
}
