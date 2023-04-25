use std::{collections::HashMap, path::PathBuf};

use ethereum_types::H160;

use rocksdb::{prelude::*, Direction, IteratorMode, OptimisticTransactionDB};
use smt_rocksdb_store::default_store::DefaultStoreMultiTree;
use sparse_merkle_tree::{error::Error, traits::Value, H256};

use crate::{
    traits::smt::StakeSmtStorage,
    types::smt::{Amount, DefaultStoreMultiSMT, Epoch, LeafValue, Proof, Root, Staker},
};

pub struct StakerSmtManager {
    db: OptimisticTransactionDB,
}

impl StakerSmtManager {
    const TOP_SMT_PREFIX: &[u8] = "top".as_bytes();

    fn new(path: PathBuf) -> Self {
        let db = OptimisticTransactionDB::open_default(path).unwrap();
        Self { db }
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

    fn update(&self, prefix: &[u8], kvs: Vec<(H256, LeafValue)>) -> Result<(), Error> {
        let tx = self.db.transaction_default();
        let mut smt =
            DefaultStoreMultiSMT::new_with_store(DefaultStoreMultiTree::new(prefix, &tx)).unwrap();
        smt.update_all(kvs).expect("update_all error");
        tx.commit().expect("db commit error");
        Ok(())
    }
}

impl StakeSmtStorage for StakerSmtManager {
    fn insert(&self, epoch: Epoch, staker_infos: Vec<(Staker, Amount)>) -> Result<(), Error> {
        let kvs = staker_infos
            .into_iter()
            .map(|(k, v)| (Self::compute_sub_smt_key(k), Self::compute_sub_smt_value(v)))
            .collect();

        self.update(&epoch.to_le_bytes(), kvs)?;

        let root = self.get_sub_root(epoch)?.unwrap().into();
        let top_kvs = vec![(Self::compute_top_smt_key(epoch), LeafValue(root))];

        self.update(Self::TOP_SMT_PREFIX, top_kvs)
    }

    fn remove(&self, epoch: Epoch, staker: Staker) -> Result<(), Error> {
        let kvs = vec![(Self::compute_sub_smt_key(staker), LeafValue::zero())];

        self.update(&epoch.to_le_bytes(), kvs)?;

        let root = self.get_sub_root(epoch)?.unwrap().into();
        let top_kvs = vec![(Self::compute_top_smt_key(epoch), LeafValue(root))];

        self.update(Self::TOP_SMT_PREFIX, top_kvs)
    }

    fn remove_batch(&self, epoch: Epoch, stakers: Vec<Staker>) -> Result<(), Error> {
        let kvs = stakers
            .into_iter()
            .map(|k| (Self::compute_sub_smt_key(k), LeafValue::zero()))
            .collect();

        self.update(&epoch.to_le_bytes(), kvs)?;

        let root = self.get_sub_root(epoch)?.unwrap().into();
        let top_kvs = vec![(Self::compute_top_smt_key(epoch), LeafValue(root))];

        self.update(Self::TOP_SMT_PREFIX, top_kvs)
    }

    fn get_amount(&self, epoch: Epoch, staker: Staker) -> Result<Option<Amount>, Error> {
        let snapshot = self.db.snapshot();
        let binding = epoch.to_le_bytes();
        let smt = DefaultStoreMultiSMT::new_with_store(DefaultStoreMultiTree::<_, ()>::new(
            &binding,
            &snapshot,
        ))
        .unwrap();

        let leaf_value = smt.get(&Self::compute_sub_smt_key(staker))?;
        if leaf_value == LeafValue::zero() {
            return Ok(None);
        }

        Ok(Some(Self::reconstruct_sub_smt_value(leaf_value)))
    }

    fn get_sub_leaves(&self, epoch: Epoch) -> Result<HashMap<Staker, Amount>, Error> {
        let mut hash_map = HashMap::new();

        let prefix = &epoch.to_le_bytes();
        let prefix_len = prefix.len();
        let snapshot = self.db.snapshot();
        let kvs: Vec<(Staker, Amount)> = snapshot
            .iterator(IteratorMode::From(prefix, Direction::Forward))
            .take_while(|(k, _)| k.starts_with(prefix))
            .filter_map(|(k, v)| {
                let leaf_key: [u8; 32] = k[prefix_len..].try_into().expect("checked 32 bytes");
                let leaf_value: [u8; 32] = v[..].try_into().expect("checked 32 bytes");
                Some((
                    Self::reconstruct_sub_smt_key(leaf_key.into()),
                    Self::reconstruct_sub_smt_value(LeafValue(leaf_value)),
                ))
            })
            .collect();

        for (k, v) in kvs.into_iter() {
            hash_map.insert(k, v);
        }

        Ok(hash_map)
    }

    fn get_sub_root(&self, epoch: Epoch) -> Result<Option<Root>, Error> {
        let snapshot = self.db.snapshot();
        let binding = epoch.to_le_bytes();
        let smt = DefaultStoreMultiSMT::new_with_store(DefaultStoreMultiTree::<_, ()>::new(
            &binding,
            &snapshot,
        ))
        .unwrap();

        Ok(Some(smt.root().clone()))
    }

    fn get_sub_roots(&self, epochs: Vec<Epoch>) -> Result<HashMap<Epoch, Option<Root>>, Error> {
        let mut hash_map = HashMap::new();

        for epoch in epochs {
            let root = self.get_sub_root(epoch)?;
            hash_map.insert(epoch, root);
        }

        Ok(hash_map)
    }

    fn get_top_root(&self) -> Result<Option<Root>, Error> {
        let snapshot = self.db.snapshot();
        let smt = DefaultStoreMultiSMT::new_with_store(DefaultStoreMultiTree::<_, ()>::new(
            Self::TOP_SMT_PREFIX,
            &snapshot,
        ))
        .unwrap();

        Ok(Some(smt.root().clone()))
    }

    fn generate_sub_proof(&self, epoch: Epoch, stakers: Vec<Staker>) -> Result<Proof, Error> {
        let keys = stakers
            .into_iter()
            .map(|k| Self::compute_sub_smt_key(k))
            .collect::<Vec<H256>>();
        let snapshot = self.db.snapshot();
        let binding = epoch.to_le_bytes();
        let rocksdb_store_smt = DefaultStoreMultiSMT::new_with_store(
            DefaultStoreMultiTree::<_, ()>::new(&binding, &snapshot),
        )
        .unwrap();

        let proof = rocksdb_store_smt
            .merkle_proof(keys.clone())?
            .compile(keys)?;
        Ok(proof.into())
    }

    fn generate_top_proof(&self, epochs: Vec<Epoch>) -> Result<Proof, Error> {
        let keys = epochs
            .into_iter()
            .map(|k| Self::compute_top_smt_key(k))
            .collect::<Vec<H256>>();
        let snapshot = self.db.snapshot();
        let rocksdb_store_smt = DefaultStoreMultiSMT::new_with_store(
            DefaultStoreMultiTree::<_, ()>::new(Self::TOP_SMT_PREFIX, &snapshot),
        )
        .unwrap();

        let proof = rocksdb_store_smt
            .merkle_proof(keys.clone())?
            .compile(keys)?;
        Ok(proof.into())
    }
}
