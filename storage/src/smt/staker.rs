use std::{collections::HashMap, path::PathBuf};

use ethereum_types::H160;

use rocksdb::{prelude::*, Direction, IteratorMode, OptimisticTransactionDB};
use smt_rocksdb_store::default_store::DefaultStoreMultiTree;
use sparse_merkle_tree::{error::Error, traits::Value, H256};

use crate::{
    traits::smt::StakeSmtStorage,
    types::smt::{Address, Amount, DefaultStoreMultiSMT, Epoch, LeafValue, Proof, Root, Staker},
};

pub struct StakerSmtManager {
    db: OptimisticTransactionDB,
}

/// SMT storage for stakers
/// For sub smt, the key is the staker address, the value is the amount of
/// staking For top smt, the key is the epoch, the value is the root of sub smt
///                          Staker Root
///                /                             \
///          epoch 1 root                   epoch 2 root
///         /      |      \                /      |      \
///    staker1  staker2  staker3       staker1  staker3  staker4
///    amount1  amount2  amount3       amount1  amount3  amount4
impl StakerSmtManager {
    const TOP_SMT_PREFIX: &[u8] = "top".as_bytes();

    pub fn new(path: PathBuf) -> Self {
        let db = OptimisticTransactionDB::open_default(path).unwrap();
        Self { db }
    }

    fn compute_sub_smt_key(key: H160) -> H256 {
        let mut buf = [0u8; 32];
        buf[..20].copy_from_slice(key.as_fixed_bytes());
        buf.into()
    }

    fn reconstruct_sub_smt_key(key: H256) -> Address {
        let mut buf = [0u8; 16];
        let key_bytes = <[u8; 32]>::from(key);
        buf.copy_from_slice(&key_bytes[..16]);
        Address::from_slice(&buf)
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
    fn insert(&self, epoch: Epoch, amounts: Vec<(Address, Amount)>) -> Result<(), Error> {
        let kvs = amounts
            .into_iter()
            .map(|(k, v)| (Self::compute_sub_smt_key(k), v.into()))
            .collect();

        self.update(&epoch.to_le_bytes(), kvs)?;

        let root = self.get_sub_root(epoch)?.unwrap().into();
        let top_kvs = vec![(Self::compute_top_smt_key(epoch), LeafValue(root))];

        self.update(Self::TOP_SMT_PREFIX, top_kvs)
    }

    fn remove(&self, epoch: Epoch, address: Address) -> Result<(), Error> {
        let kvs = vec![(Self::compute_sub_smt_key(address), LeafValue::zero())];

        self.update(&epoch.to_le_bytes(), kvs)?;

        let root = self.get_sub_root(epoch)?.unwrap().into();
        let top_kvs = vec![(Self::compute_top_smt_key(epoch), LeafValue(root))];

        self.update(Self::TOP_SMT_PREFIX, top_kvs)
    }

    fn remove_batch(&self, epoch: Epoch, addresses: Vec<Address>) -> Result<(), Error> {
        let kvs = addresses
            .into_iter()
            .map(|k| (Self::compute_sub_smt_key(k), LeafValue::zero()))
            .collect();

        self.update(&epoch.to_le_bytes(), kvs)?;

        let root = self.get_sub_root(epoch)?.unwrap().into();
        let top_kvs = vec![(Self::compute_top_smt_key(epoch), LeafValue(root))];

        self.update(Self::TOP_SMT_PREFIX, top_kvs)
    }

    fn get_amount(&self, epoch: Epoch, address: Address) -> Result<Option<Amount>, Error> {
        let snapshot = self.db.snapshot();
        let binding = epoch.to_le_bytes();
        let smt = DefaultStoreMultiSMT::new_with_store(DefaultStoreMultiTree::<_, ()>::new(
            &binding, &snapshot,
        ))
        .unwrap();

        let leaf_value = smt.get(&Self::compute_sub_smt_key(address))?;
        if leaf_value == LeafValue::zero() {
            return Ok(None);
        }

        Ok(Some(Amount::from(leaf_value)))
    }

    fn get_sub_leaves(&self, epoch: Epoch) -> Result<HashMap<Address, Amount>, Error> {
        let mut hash_map = HashMap::new();

        let prefix = &epoch.to_le_bytes();
        let prefix_len = prefix.len();
        let key_len = prefix_len + 32;
        let snapshot = self.db.snapshot();
        let kvs: Vec<(Staker, Amount)> = snapshot
            .iterator(IteratorMode::From(prefix, Direction::Forward))
            .take_while(|(k, _)| k.starts_with(prefix))
            .filter_map(|(k, v)| {
                if key_len != key_len {
                    None
                } else {
                    let leaf_key: [u8; 32] = k[prefix_len..].try_into().expect("checked 32 bytes");
                    let leaf_value: [u8; 32] = v[..].try_into().expect("checked 32 bytes");
                    Some((
                        Self::reconstruct_sub_smt_key(leaf_key.into()),
                        Amount::from(LeafValue(leaf_value)),
                    ))
                }
            })
            .collect();

        for (k, v) in kvs.into_iter() {
            hash_map.insert(k, v);
        }

        Ok(hash_map)
    }

    fn get_sub_root(&self, epoch: Epoch) -> Result<Option<Root>, Error> {
        let snapshot = self.db.snapshot();
        let prefix = epoch.to_le_bytes();
        let smt = DefaultStoreMultiSMT::new_with_store(DefaultStoreMultiTree::<_, ()>::new(
            &prefix, &snapshot,
        ))
        .unwrap();

        Ok(Some(smt.root().clone()))
    }

    fn get_sub_roots(&self, epochs: Vec<Epoch>) -> Result<HashMap<Epoch, Option<Root>>, Error> {
        let mut hash_map = HashMap::new();
        let snapshot = self.db.snapshot();

        for epoch in epochs {
            let prefix = epoch.to_le_bytes();
            let smt = DefaultStoreMultiSMT::new_with_store(DefaultStoreMultiTree::<_, ()>::new(
                &prefix, &snapshot,
            ))
            .unwrap();
            hash_map.insert(epoch, Some(smt.root().clone()));
        }

        Ok(hash_map)
    }

    fn get_top_root(&self) -> Result<Root, Error> {
        let snapshot = self.db.snapshot();
        let smt = DefaultStoreMultiSMT::new_with_store(DefaultStoreMultiTree::<_, ()>::new(
            Self::TOP_SMT_PREFIX,
            &snapshot,
        ))
        .unwrap();

        Ok(smt.root().clone())
    }

    fn generate_sub_proof(&self, epoch: Epoch, addresses: Vec<Address>) -> Result<Proof, Error> {
        let keys = addresses
            .into_iter()
            .map(|k| Self::compute_sub_smt_key(k))
            .collect::<Vec<H256>>();
        let snapshot = self.db.snapshot();
        let binding = epoch.to_le_bytes();
        let smt = DefaultStoreMultiSMT::new_with_store(DefaultStoreMultiTree::<_, ()>::new(
            &binding, &snapshot,
        ))
        .unwrap();

        Ok(smt.merkle_proof(keys.clone())?.compile(keys)?.into())
    }

    fn generate_top_proof(&self, epochs: Vec<Epoch>) -> Result<Proof, Error> {
        let keys = epochs
            .into_iter()
            .map(|k| Self::compute_top_smt_key(k))
            .collect::<Vec<H256>>();
        let snapshot = self.db.snapshot();
        let smt = DefaultStoreMultiSMT::new_with_store(DefaultStoreMultiTree::<_, ()>::new(
            Self::TOP_SMT_PREFIX,
            &snapshot,
        ))
        .unwrap();

        Ok(smt.merkle_proof(keys.clone())?.compile(keys)?.into())
    }
}
