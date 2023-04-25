use std::{collections::HashMap, path::PathBuf, sync::Arc};

use dashmap::DashMap;

use sparse_merkle_tree::error::Error;

use crate::{
    smt::staker::StakerSmtManager,
    traits::smt::{DelegateSmtStorage, StakeSmtStorage},
    types::smt::{Amount, Delegator, Epoch, Proof, Root, Staker},
};

pub struct DelegatorSmtManager {
    dbs:  DashMap<Staker, Arc<StakerSmtManager>>,
    path: PathBuf,
}

impl DelegatorSmtManager {
    pub fn new(path: PathBuf) -> Self {
        Self {
            dbs: DashMap::new(),
            path,
        }
    }
}

impl DelegateSmtStorage for DelegatorSmtManager {
    fn insert(
        &self,
        epoch: Epoch,
        delegators: HashMap<Staker, Vec<(Delegator, Amount)>>,
    ) -> Result<(), Error> {
        for (staker, delegator_infos) in delegators {
            let mut current_path = self.path.clone();
            current_path.push(staker.to_string());

            let db = self
                .dbs
                .entry(staker)
                .or_insert_with(|| Arc::new(StakerSmtManager::new(current_path)));

            db.insert(epoch, delegator_infos)?;
        }

        Ok(())
    }

    fn remove(
        &self,
        epoch: Epoch,
        delegators: HashMap<Staker, Vec<Delegator>>,
    ) -> Result<(), Error> {
        for (staker, delegator_addresses) in delegators {
            let db = self.dbs.get(&staker).unwrap();
            db.remove_batch(epoch, delegator_addresses)?;
        }

        Ok(())
    }

    fn get_amount(
        &self,
        delegator: Delegator,
        staker: Staker,
        epoch: Epoch,
    ) -> Result<Option<Amount>, Error> {
        self.dbs.get(&staker).unwrap().get_amount(epoch, delegator)
    }

    fn get_sub_leaves(
        &self,
        staker: Staker,
        epoch: Epoch,
    ) -> Result<HashMap<Delegator, Amount>, Error> {
        self.dbs.get(&staker).unwrap().get_sub_leaves(epoch)
    }

    fn get_sub_root(&self, staker: Staker, epoch: Epoch) -> Result<Option<Root>, Error> {
        self.dbs.get(&staker).unwrap().get_sub_root(epoch)
    }

    fn get_sub_roots(
        &self,
        staker: Staker,
        epochs: Vec<Epoch>,
    ) -> Result<HashMap<Epoch, Option<Root>>, Error> {
        self.dbs.get(&staker).unwrap().get_sub_roots(epochs)
    }

    fn get_top_root(&self, staker: Staker) -> Result<Root, Error> {
        self.dbs.get(&staker).unwrap().get_top_root()
    }

    fn get_top_roots(&self, stakers: Vec<Staker>) -> Result<HashMap<Staker, Root>, Error> {
        let mut hash_map = HashMap::new();
        for staker in stakers {
            hash_map.insert(staker, self.dbs.get(&staker).unwrap().get_top_root()?);
        }

        Ok(hash_map)
    }

    fn generate_sub_proof(
        &self,
        staker: Staker,
        epoch: Epoch,
        delegators: Vec<Delegator>,
    ) -> Result<Proof, Error> {
        self.dbs
            .get(&staker)
            .unwrap()
            .generate_sub_proof(epoch, delegators)
    }

    fn generate_top_proof(&self, staker: Staker, epochs: Vec<Epoch>) -> Result<Proof, Error> {
        self.dbs.get(&staker).unwrap().generate_top_proof(epochs)
    }
}
