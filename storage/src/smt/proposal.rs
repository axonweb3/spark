use std::{collections::HashMap, path::PathBuf};

use ethereum_types::H160;

use rocksdb::{prelude::*, Direction, IteratorMode, OptimisticTransactionDB};
use smt_rocksdb_store::default_store::DefaultStoreMultiTree;
use sparse_merkle_tree::{error::Error, H256};

use crate::{
    traits::smt::ProposalSmtStorage,
    types::smt::{
        Address, DefaultStoreMultiSMT, Epoch, LeafValue, Proof, ProposalCount, Root, Validator,
    },
};

pub struct ProposalSmtManager {
    db: OptimisticTransactionDB,
}

impl ProposalSmtManager {
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

impl ProposalSmtStorage for ProposalSmtManager {
    fn insert(
        &self,
        epoch: Epoch,
        proposals: Vec<(Validator, ProposalCount)>,
    ) -> Result<(), Error> {
        let kvs = proposals
            .into_iter()
            .map(|(k, v)| (Self::compute_sub_smt_key(k), v.into()))
            .collect();

        self.update(&epoch.to_le_bytes(), kvs)?;

        let root = self.get_sub_root(epoch)?.unwrap().into();
        let top_kvs = vec![(Self::compute_top_smt_key(epoch), LeafValue(root))];

        self.update(Self::TOP_SMT_PREFIX, top_kvs)
    }

    fn get_proposal(&self, epoch: Epoch) -> Result<HashMap<Validator, ProposalCount>, Error> {
        let mut hash_map = HashMap::new();

        let prefix = &epoch.to_le_bytes();
        let prefix_len = prefix.len();
        let key_len = prefix_len + 32;
        let snapshot = self.db.snapshot();
        let kvs: Vec<(Validator, ProposalCount)> = snapshot
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
                        ProposalCount::from(LeafValue(leaf_value)),
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

    fn generate_sub_proof(&self, epoch: Epoch, validators: Vec<Validator>) -> Result<Proof, Error> {
        let snapshot = self.db.snapshot();
        let prefix = epoch.to_le_bytes();
        let mut keys = Vec::new();
        for validator in validators {
            keys.push(Self::compute_sub_smt_key(validator));
        }

        let smt = DefaultStoreMultiSMT::new_with_store(DefaultStoreMultiTree::<_, ()>::new(
            &prefix, &snapshot,
        ))
        .unwrap();

        Ok(smt.merkle_proof(keys.clone())?.compile(keys)?.into())
    }

    fn generate_top_proof(&self, epochs: Vec<Epoch>) -> Result<Proof, Error> {
        let snapshot = self.db.snapshot();
        let mut keys = Vec::new();
        for epoch in epochs {
            keys.push(Self::compute_top_smt_key(epoch));
        }

        let smt = DefaultStoreMultiSMT::new_with_store(DefaultStoreMultiTree::<_, ()>::new(
            Self::TOP_SMT_PREFIX,
            &snapshot,
        ))
        .unwrap();

        Ok(smt.merkle_proof(keys.clone())?.compile(keys)?.into())
    }
}
