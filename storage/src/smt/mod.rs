use std::{collections::HashMap, sync::Arc};

use sparse_merkle_tree::{error::Error, blake2b::Blake2bHasher, H256};

use crate::{types::smt::{SmtType, Address, Amount, Leaf, Root, new_blake2b}, traits::smt::SmtStorage};

#[derive(Default)]
pub struct Smt {
    smt: SmtType,
}

impl Smt {
    pub fn new() -> Self {
        let smt = SmtType::default();
        Self { smt }
    }

    fn compute_smt_key(address: Address) -> H256 {
        let mut buf = [0u8; 32];
        let mut hasher = new_blake2b();
        hasher.update(&address.as_bytes());
        hasher.finalize(&mut buf);
        buf.into()
    }
}

impl SmtStorage for Smt {
    fn insert(&mut self, key: Address, value: Amount) -> Result<(), Error> {
        self.smt.update(Self::compute_smt_key(key), value)?;
        Ok(())
    }

    fn get(&self, key: Address) -> Result<Option<Amount>, Error> {
        let value = self.smt.get(&Self::compute_smt_key(key))?;
        if value == Amount::default() {
            return Ok(None);
        }
        Ok(Some(value))
    }

    fn remove(&mut self, key: Address) -> Result<(), Error> {
        self.smt
            .update(Self::compute_smt_key(key), Amount::default())?;
        Ok(())
    }

    fn compute_root(&self, leaves: Vec<Leaf>) -> Result<Root, Error> {
        let keys = leaves.iter().map(|(k, _)| *k).collect();
        let root = self
            .smt
            .merkle_proof(keys)?
            .compute_root::<Blake2bHasher>(leaves)?;
        Ok(root)
    }

    fn verify_root(&self, root_hash: H256, leaves: Vec<Leaf>) -> Result<bool, Error> {
        let keys = leaves.iter().map(|(k, _)| *k).collect();
        self.smt
            .merkle_proof(keys)?
            .verify::<Blake2bHasher>(&root_hash, leaves)
    }

    fn save_db(&self, db: rocksdb::DB) -> Result<(), Error> {
        todo!()
    }

    fn root(&self) -> Result<H256, Error> {
        Ok(*self.smt.root())
    }
}


#[derive(Default)]
pub struct SmtMap {
    smts: HashMap<Address, Arc<Smt>>,
}

impl SmtMap {
    pub fn new(keys: Vec<Address>) -> Self {
        let mut smts = HashMap::new();
        for key in keys {
            let smt = Smt::new();
            smts.insert(key, Arc::new(smt));
        }
        Self { smts }
    }
}

#[derive(Default)]
pub struct StakerSmtManager {
    smt_map: SmtMap,
}

#[derive(Default)]
pub struct DelegatorSmtManager {
    smt_map: SmtMap,
}

#[derive(Default)]
pub struct RewardSmtManager {
    smt_map: SmtMap,
}

#[derive(Default)]
pub struct ExpiredRewardSmtManager {
    smt_map: SmtMap,
}
