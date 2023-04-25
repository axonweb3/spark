use std::path::PathBuf;

use rocksdb::{prelude::*, OptimisticTransactionDB};
use smt_rocksdb_store::default_store::DefaultStore;
use sparse_merkle_tree::{error::Error, traits::Value, H256};

use crate::{
    traits::smt::RewardSmtStorage,
    types::smt::{Address, DefaultStoreSMT, Epoch, LeafValue, Proof, Root},
};

pub struct RewardSmtManager {
    db: OptimisticTransactionDB,
}

impl RewardSmtManager {
    pub fn new(path: PathBuf) -> Self {
        let db = OptimisticTransactionDB::open_default(path).unwrap();

        Self { db }
    }

    fn compute_smt_key(address: Address) -> H256 {
        let mut key = [0u8; 32];
        key[0..20].copy_from_slice(&address.as_bytes());
        key.into()
    }

    fn compute_smt_value(epoch: Epoch) -> LeafValue {
        let mut value = [0u8; 32];
        value[0..8].copy_from_slice(&epoch.to_le_bytes());
        LeafValue(value)
    }

    fn reconstruct_sub_smt_value(value: LeafValue) -> Epoch {
        let mut epoch_bytes = [0u8; 8];
        epoch_bytes.copy_from_slice(&value.0[0..8]);
        Epoch::from_le_bytes(epoch_bytes)
    }
}

impl RewardSmtStorage for RewardSmtManager {
    fn insert(&self, address: Address, epoch: Epoch) -> Result<(), Error> {
        let kvs = vec![(
            Self::compute_smt_key(address),
            Self::compute_smt_value(epoch),
        )];

        let tx = self.db.transaction_default();
        let mut smt = DefaultStoreSMT::new_with_store(DefaultStore::new(&tx)).unwrap();
        smt.update_all(kvs).expect("update_all error");
        tx.commit().expect("db commit error");
        Ok(())
    }

    fn get_root(&self) -> Result<Root, Error> {
        let snapshot = self.db.snapshot();
        let smt = DefaultStoreSMT::new_with_store(DefaultStore::<_, ()>::new(&snapshot)).unwrap();

        Ok(smt.root().clone())
    }

    fn get_latest_reward_epoch(&self, address: Address) -> Result<Option<Epoch>, Error> {
        let snapshot = self.db.snapshot();
        let smt = DefaultStoreSMT::new_with_store(DefaultStore::<_, ()>::new(&snapshot)).unwrap();

        let leaf_value = smt.get(&Self::compute_smt_key(address))?;
        if leaf_value == LeafValue::zero() {
            return Ok(None);
        }

        Ok(Some(Self::reconstruct_sub_smt_value(leaf_value)))
    }

    fn generate_proof(&self, addresses: Vec<Address>) -> Result<Proof, Error> {
        let snapshot = self.db.snapshot();
        let smt = DefaultStoreSMT::new_with_store(DefaultStore::<_, ()>::new(&snapshot)).unwrap();

        let mut keys = Vec::new();
        for address in addresses {
            keys.push(Self::compute_smt_key(address));
        }

        Ok(smt.merkle_proof(keys.clone())?.compile(keys)?.into())
    }
}
