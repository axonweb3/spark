use std::{collections::HashMap, hash::Hash, sync::Arc};

use ethereum_types::H160;
use parking_lot::Mutex;

use sparse_merkle_tree::{blake2b::Blake2bHasher, error::Error, traits::Value, H256};

use crate::{
    traits::smt::{SmtMapStorage, SmtStorage, StakeSmtStorage},
    types::smt::{Amount, Epoch, LeafValue, Proof, Root, SmtType, Staker},
};

#[derive(Default)]
pub struct Smt {
    smt: SmtType,
}

impl Smt {
    pub fn new() -> Self {
        let smt = SmtType::default();
        Self { smt }
    }
}

impl SmtStorage for Smt {
    fn insert(&mut self, key: H256, value: LeafValue) -> Result<(), Error> {
        self.smt.update(key, value)?;
        Ok(())
    }

    fn get(&self, key: H256) -> Result<Option<LeafValue>, Error> {
        let value = self.smt.get(&key)?;
        if value == LeafValue::default() {
            return Ok(None);
        }
        Ok(Some(value))
    }

    fn get_leaves(&self) -> Result<HashMap<H256, LeafValue>, Error> {
        Ok(self.smt.store().leaves_map().clone())
    }

    fn remove(&mut self, key: H256) -> Result<(), Error> {
        self.smt.update(key, LeafValue::default())?;
        Ok(())
    }

    fn compute_root(&self, leaves: Vec<(H256, LeafValue)>) -> Result<Root, Error> {
        let keys = leaves.iter().map(|(k, _)| *k).collect();
        let converted_leaves = leaves.iter().map(|(k, v)| (*k, v.to_h256())).collect();
        let root = self
            .smt
            .merkle_proof(keys)?
            .compute_root::<Blake2bHasher>(converted_leaves)?;
        Ok(root)
    }

    fn verify_root(&self, root_hash: H256, leaves: Vec<(H256, LeafValue)>) -> Result<bool, Error> {
        let keys = leaves.iter().map(|(k, _)| *k).collect();
        let converted_leaves = leaves.iter().map(|(k, v)| (*k, v.to_h256())).collect();
        self.smt
            .merkle_proof(keys)?
            .verify::<Blake2bHasher>(&root_hash, converted_leaves)
    }

    fn generate_proof(&self, leaves_keys: Vec<H256>) -> Result<Proof, Error> {
        let compiled_proof = self
            .smt
            .merkle_proof(leaves_keys.clone())?
            .compile(leaves_keys)?;
        Ok(compiled_proof.into())
    }

    fn save_db(&self, db: rocksdb::DB) -> Result<(), Error> {
        todo!()
    }

    fn root(&self) -> Result<Root, Error> {
        Ok(*self.smt.root())
    }
}

#[derive(Default)]
pub struct SmtMap<K> {
    smts: HashMap<K, Arc<Mutex<Smt>>>,
}

impl<K: Eq + Clone + Hash> SmtMap<K> {
    pub fn new(keys: Vec<K>) -> Self {
        let mut smts = HashMap::new();
        for key in keys {
            let smt = Smt::new();
            smts.insert(key, Arc::new(Mutex::new(smt)));
        }
        Self { smts }
    }
}

impl<K: Eq + Clone + Hash> SmtMapStorage<K> for SmtMap<K> {
    fn get_smt(&self, key: K) -> Option<Arc<Mutex<Smt>>> {
        self.smts.get(&key).cloned()
    }

    fn get_root(&self, key: K) -> Result<Option<Root>, Error> {
        if let Some(smt) = self.smts.get(&key) {
            return Ok(Some(smt.lock().root()?));
        }
        Ok(None)
    }

    fn get_roots(&self, keys: Vec<K>) -> Result<HashMap<K, Option<Root>>, Error> {
        let mut hash_map = HashMap::new();
        for key in keys {
            if let Some(smt) = self.smts.get(&key) {
                let root = smt.lock().root()?;
                hash_map.insert(key, Some(root));
            } else {
                hash_map.insert(key, None);
            }
        }

        Ok(hash_map)
    }

    fn insert_smt(&mut self, key: K, smt: Smt) -> Result<(), Error> {
        self.smts.insert(key, Arc::new(Mutex::new(smt)));
        Ok(())
    }

    fn remove_smt(&mut self, key: K) -> Result<(), Error> {
        self.smts.remove(&key);
        Ok(())
    }

    fn load(&self, keys: Vec<K>) -> Result<Smt, Error> {
        todo!()
    }
}

#[derive(Default)]
pub struct StakerSmtManager {
    sub_smt_map: SmtMap<Epoch>,
    top_smt:     Smt,
}

impl StakerSmtManager {
    fn new(keys: Vec<Epoch>) -> Self {
        let sub_smt_map = SmtMap::new(keys);
        let top_smt = Smt::new();
        Self {
            sub_smt_map,
            top_smt,
        }
    }

    fn compute_sub_smt_key(key: H160) -> H256 {
        let mut buf = [0u8; 32];
        buf[..20].copy_from_slice(key.as_fixed_bytes());
        buf.into()
    }

    fn reconstruct_sub_smt_key(key: H256) -> Staker {
        let mut buf = [0u8; 16];
        let key_bytes = <[u8; 32]>::from(key);
        buf.copy_from_slice(&key_bytes[..16]);
        Staker::from_slice(&buf)
    }

    fn compute_sub_smt_value(amount: Amount) -> LeafValue {
        let bytes = amount.to_le_bytes();
        let mut buf = [0u8; 32];
        buf[..16].copy_from_slice(&bytes);
        LeafValue(buf)
    }

    fn reconstruct_sub_smt_value(leaf_value: LeafValue) -> Amount {
        let mut amount_bytes = [0u8; 16];
        amount_bytes.copy_from_slice(&leaf_value.0[..16]);
        Amount::from_le_bytes(amount_bytes)
    }

    fn compute_top_smt_key(key: u64) -> H256 {
        let mut buf = [0u8; 32];
        buf[..8].copy_from_slice(&key.to_le_bytes());
        buf.into()
    }
}

impl StakeSmtStorage for StakerSmtManager {
    fn insert(&mut self, epoch: Epoch, staker_infos: Vec<(Staker, Amount)>) -> Result<(), Error> {
        let mut smt = Smt::new();
        for (staker, amount) in staker_infos {
            smt.insert(
                Self::compute_sub_smt_key(staker),
                Self::compute_sub_smt_value(amount),
            )?;
        }
        self.top_smt.insert(
            Self::compute_top_smt_key(epoch),
            LeafValue(smt.root()?.into()),
        )?;
        self.sub_smt_map.insert_smt(epoch, smt)
    }

    fn remove(&mut self, epoch: Epoch, staker: Staker) -> Result<(), Error> {
        let smt = self.sub_smt_map.get_smt(epoch).unwrap();
        smt.lock().remove(Self::compute_sub_smt_key(staker))?;
        Ok(())
    }

    fn remove_batch(&mut self, epoch: Epoch, stakers: Vec<Staker>) -> Result<(), Error> {
        let smt = self.sub_smt_map.get_smt(epoch).unwrap();
        for staker in stakers {
            smt.lock().remove(Self::compute_sub_smt_key(staker))?;
        }

        self.top_smt.insert(
            Self::compute_top_smt_key(epoch),
            LeafValue(smt.lock().root()?.into()),
        )?;
        Ok(())
    }

    fn get_amount(&self, staker: Staker, epoch: Epoch) -> Result<Option<Amount>, Error> {
        let smt = self.sub_smt_map.get_smt(epoch).unwrap();
        if let Some(leaf_value) = smt.lock().get(Self::compute_sub_smt_key(staker))? {
            return Ok(Some(Self::reconstruct_sub_smt_value(leaf_value)));
        }

        Ok(None)
    }

    fn get_sub_leaves(&self, epoch: Epoch) -> Result<HashMap<Staker, Amount>, Error> {
        let smt = self.sub_smt_map.get_smt(epoch).unwrap();
        let leaves = smt.lock().get_leaves()?;
        let mut hash_map = HashMap::new();
        for (k, v) in leaves {
            let staker = Self::reconstruct_sub_smt_key(k);
            let amount = Self::reconstruct_sub_smt_value(v);
            hash_map.insert(staker, amount);
        }
        Ok(hash_map)
    }

    fn get_sub_root(&self, epoch: Epoch) -> Result<Option<Root>, Error> {
        self.sub_smt_map.get_root(epoch)
    }

    fn get_sub_roots(&self, epochs: Vec<Epoch>) -> Result<HashMap<Epoch, Option<Root>>, Error> {
        self.sub_smt_map.get_roots(epochs)
    }

    fn get_top_root(&self) -> Result<Root, Error> {
        self.top_smt.root()
    }

    fn generate_sub_proof(&self, epoch: Epoch, stakers: Vec<Staker>) -> Result<Proof, Error> {
        let keys = stakers
            .iter()
            .map(|s| Self::compute_sub_smt_key(*s))
            .collect();
        self.sub_smt_map
            .get_smt(epoch)
            .unwrap()
            .lock()
            .generate_proof(keys)
    }

    fn generate_top_proof(&self, epochs: Vec<Epoch>) -> Result<Proof, Error> {
        let keys = epochs
            .iter()
            .map(|e| Self::compute_top_smt_key(*e))
            .collect();
        self.top_smt.generate_proof(keys)
    }
}
