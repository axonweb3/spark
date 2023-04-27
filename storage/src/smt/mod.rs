use std::{collections::HashMap, path::PathBuf, sync::Arc, vec};

use async_trait::async_trait;

use rocksdb::{prelude::*, Direction, IteratorMode, OptimisticTransactionDB};
use smt_rocksdb_store::cf_store::{ColumnFamilyStore, ColumnFamilyStoreMultiTree};
use sparse_merkle_tree::{error::Error, traits::Value, H256};

use crate::{
    traits::smt::{DelegateSmtStorage, ProposalSmtStorage, RewardSmtStorage, StakeSmtStorage},
    types::smt::*,
};

macro_rules! create_table_cfs {
    ($db: expr, $cf: expr) => {{
        let cf1 = format!("{}_{}", $cf.to_string(), "branch");
        let cf2 = format!("{}_{}", $cf.to_string(), "leaf");

        let cf_opts = Options::default();
        $db.create_cf(cf1, &cf_opts).unwrap();
        $db.create_cf(cf2, &cf_opts).unwrap();
    }};
}

macro_rules! get_smt {
    ($db: expr, $cf: expr, $prefix: expr, $inner: expr) => {{
        let cf1 = format!("{}_{}", $cf.to_string(), "branch");
        let cf2 = format!("{}_{}", $cf.to_string(), "leaf");

        let cf1_handle = $db.cf_handle(&cf1).unwrap();
        let cf2_handle = $db.cf_handle(&cf2).unwrap();

        let smt = ColumnFamilyStoreMultiSMT::new_with_store(
            ColumnFamilyStoreMultiTree::<_, ()>::new($prefix, $inner, cf1_handle, cf2_handle),
        )
        .unwrap();

        smt
    }};

    ($db: expr, $cf: expr, $inner: expr) => {{
        let cf1 = format!("{}_{}", $cf.to_string(), "branch");
        let cf2 = format!("{}_{}", $cf.to_string(), "leaf");

        let cf1_handle = $db.cf_handle(&cf1).unwrap();
        let cf2_handle = $db.cf_handle(&cf2).unwrap();

        let smt = ColumnFamilyStoreSMT::new_with_store(ColumnFamilyStore::<_, ()>::new(
            $inner, cf1_handle, cf2_handle,
        ))
        .unwrap();

        smt
    }};
}

pub struct SmtManager {
    db: Arc<OptimisticTransactionDB>,
}

impl SmtManager {
    async fn get_sub_leaves(
        &self,
        prefix: &[u8],
        table: &str,
    ) -> Result<HashMap<Address, Amount>, Error> {
        let mut hash_map = HashMap::new();

        let prefix_len = prefix.len();
        let key_len = prefix_len + 32;
        let mode = IteratorMode::From(&prefix, Direction::Forward);
        let read_opt = ReadOptions::default();
        let cf = self.db.cf_handle(&format!("{}_{}", table, "leaf")).unwrap();
        let cf_iter = self.db.get_iter_cf(cf, &read_opt, mode).unwrap();
        let kvs = cf_iter
            .into_iter()
            .take_while(|(k, _)| k.starts_with(&prefix))
            .filter_map(|(k, v)| {
                if key_len != key_len {
                    None
                } else {
                    let leaf_key: [u8; 32] = k[prefix_len..].try_into().expect("checked 32 bytes");
                    let leaf_value: [u8; 32] = v[..].try_into().expect("checked 32 bytes");
                    Some((
                        Address::from_slice(&SmtKeyDecode::Address(leaf_key).from_h256()),
                        Amount::from(LeafValue(leaf_value)),
                    ))
                }
            })
            .collect::<Vec<(Staker, Amount)>>();

        for (k, v) in kvs.into_iter() {
            hash_map.insert(k, v);
        }

        Ok(hash_map)
    }
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
impl SmtManager {
    pub fn new(path: PathBuf) -> Self {
        let mut db_opts = Options::default();
        db_opts.create_missing_column_families(true);
        db_opts.create_if_missing(true);
        let mut db = OptimisticTransactionDB::open(&db_opts, path).unwrap();

        create_table_cfs!(db, STAKER_TABLE);
        create_table_cfs!(db, DELEGATOR_TABLE);
        create_table_cfs!(db, REWARD_TABLE);
        create_table_cfs!(db, PROPOSAL_TABLE);

        Self { db: Arc::new(db) }
    }

    fn update(&self, cf: &str, prefix: &[u8], kvs: Vec<(H256, LeafValue)>) -> Result<(), Error> {
        let inner = self.db.transaction_default();
        let mut smt = get_smt!(self.db, cf, prefix, &inner);
        smt.update_all(kvs).expect("update_all error");
        inner.commit().expect("db commit error");
        Ok(())
    }
}

#[async_trait]
impl StakeSmtStorage for SmtManager {
    async fn insert_stake(
        &self,
        epoch: Epoch,
        amounts: Vec<(Address, Amount)>,
    ) -> Result<(), Error> {
        let kvs = amounts
            .into_iter()
            .map(|(k, v)| {
                (
                    SmtKeyEncode::Address(k).to_h256(),
                    SmtValueEncode::Amount(v).to_leaf_value(),
                )
            })
            .collect();

        self.update(&STAKER_TABLE, &SmtPrefixType::Epoch(epoch).as_prefix(), kvs)?;

        let root = self.get_sub_root_stake(epoch).await?.unwrap();
        let top_kvs = vec![(
            SmtKeyEncode::Epoch(epoch).to_h256(),
            SmtValueEncode::Root(root).to_leaf_value(),
        )];

        self.update(&STAKER_TABLE, &SmtPrefixType::Top.as_prefix(), top_kvs)
    }

    async fn remove_stake(&self, epoch: Epoch, address: Address) -> Result<(), Error> {
        let kvs = vec![(SmtKeyEncode::Address(address).to_h256(), LeafValue::zero())];

        self.update(&STAKER_TABLE, &SmtPrefixType::Epoch(epoch).as_prefix(), kvs)?;

        let root = self.get_sub_root_stake(epoch).await?.unwrap();
        let top_kvs = vec![(
            SmtKeyEncode::Epoch(epoch).to_h256(),
            SmtValueEncode::Root(root).to_leaf_value(),
        )];

        self.update(&STAKER_TABLE, &SmtPrefixType::Top.as_prefix(), top_kvs)
    }

    async fn remove_batch_stake(&self, epoch: Epoch, addresses: Vec<Address>) -> Result<(), Error> {
        let kvs = addresses
            .into_iter()
            .map(|k| (SmtKeyEncode::Address(k).to_h256(), LeafValue::zero()))
            .collect();

        self.update(&STAKER_TABLE, &SmtPrefixType::Epoch(epoch).as_prefix(), kvs)?;

        let root = self.get_sub_root_stake(epoch).await?.unwrap();
        let top_kvs = vec![(
            SmtKeyEncode::Epoch(epoch).to_h256(),
            SmtValueEncode::Root(root).to_leaf_value(),
        )];

        self.update(&STAKER_TABLE, &SmtPrefixType::Top.as_prefix(), top_kvs)
    }

    async fn get_amount_stake(
        &self,
        epoch: Epoch,
        address: Address,
    ) -> Result<Option<Amount>, Error> {
        let prefix = SmtPrefixType::Epoch(epoch).as_prefix();
        let snapshot = self.db.snapshot();
        let smt = get_smt!(self.db, &STAKER_TABLE, &prefix, &snapshot);

        let leaf_value = smt.get(&SmtKeyEncode::Address(address).to_h256())?;
        if leaf_value == LeafValue::zero() {
            return Ok(None);
        }

        Ok(Some(Amount::from(leaf_value)))
    }

    async fn get_sub_leaves_stake(&self, epoch: Epoch) -> Result<HashMap<Address, Amount>, Error> {
        let prefix = SmtPrefixType::Epoch(epoch).as_prefix();
        self.get_sub_leaves(&prefix, &STAKER_TABLE).await
    }

    async fn get_sub_root_stake(&self, epoch: Epoch) -> Result<Option<Root>, Error> {
        let prefix = SmtPrefixType::Epoch(epoch).as_prefix();
        let snapshot = self.db.snapshot();
        let smt = get_smt!(self.db, &STAKER_TABLE, &prefix, &snapshot);

        Ok(Some(smt.root().clone()))
    }

    async fn get_sub_roots_stake(
        &self,
        epochs: Vec<Epoch>,
    ) -> Result<HashMap<Epoch, Option<Root>>, Error> {
        let mut hash_map = HashMap::new();

        for epoch in epochs {
            let root = self.get_sub_root_stake(epoch).await?;
            hash_map.insert(epoch, root);
        }

        Ok(hash_map)
    }

    async fn get_top_root_stake(&self) -> Result<Root, Error> {
        let prefix = SmtPrefixType::Top.as_prefix();
        let snapshot = self.db.snapshot();
        let smt = get_smt!(self.db, &STAKER_TABLE, &prefix, &snapshot);

        Ok(smt.root().clone())
    }

    async fn generate_sub_proof_stake(
        &self,
        epoch: Epoch,
        addresses: Vec<Address>,
    ) -> Result<Proof, Error> {
        let prefix = SmtPrefixType::Epoch(epoch).as_prefix();
        let snapshot = self.db.snapshot();
        let keys = addresses
            .into_iter()
            .map(|k| SmtKeyEncode::Address(k).to_h256())
            .collect::<Vec<H256>>();
        let smt = get_smt!(self.db, &STAKER_TABLE, &prefix, &snapshot);

        Ok(smt.merkle_proof(keys.clone())?.compile(keys)?.into())
    }

    async fn generate_top_proof_stake(&self, epochs: Vec<Epoch>) -> Result<Proof, Error> {
        let prefix = SmtPrefixType::Top.as_prefix();
        let snapshot = self.db.snapshot();
        let keys = epochs
            .into_iter()
            .map(|k| SmtKeyEncode::Epoch(k).to_h256())
            .collect::<Vec<H256>>();
        let smt = get_smt!(self.db, &STAKER_TABLE, &prefix, &snapshot);

        Ok(smt.merkle_proof(keys.clone())?.compile(keys)?.into())
    }
}

#[async_trait]
impl DelegateSmtStorage for SmtManager {
    async fn insert_delegate(
        &self,
        epoch: Epoch,
        delegators: HashMap<Staker, Vec<(Delegator, Amount)>>,
    ) -> Result<(), Error> {
        let prefix = SmtPrefixType::Epoch(epoch).as_prefix();
        for (staker, amounts) in delegators {
            let mut current_prefix = prefix.clone();
            let mut staker_prefix = SmtPrefixType::Address(staker).as_prefix();
            current_prefix.append(&mut staker_prefix.clone());
            let kvs = amounts
                .into_iter()
                .map(|(k, v)| {
                    (
                        SmtKeyEncode::Address(k).to_h256(),
                        SmtValueEncode::Amount(v).to_leaf_value(),
                    )
                })
                .collect();

            self.update(&DELEGATOR_TABLE, &current_prefix, kvs)?;

            let root = self.get_sub_root_delegate(staker, epoch).await?.unwrap();
            let top_kvs = vec![(
                SmtKeyEncode::Epoch(epoch).to_h256(),
                SmtValueEncode::Root(root).to_leaf_value(),
            )];

            let mut top_prefix = SmtPrefixType::Top.as_prefix();
            top_prefix.append(&mut staker_prefix);
            self.update(&DELEGATOR_TABLE, &top_prefix, top_kvs)?;
        }

        Ok(())
    }

    async fn remove_delegate(
        &self,
        epoch: Epoch,
        delegators: HashMap<Staker, Vec<Delegator>>,
    ) -> Result<(), Error> {
        let mut hash_map = HashMap::new();
        for (staker, addresses) in delegators {
            let kvs = addresses
                .into_iter()
                .map(|k| (k, Amount::default()))
                .collect();
            hash_map.insert(staker, kvs);
        }

        self.insert_delegate(epoch, hash_map).await
    }

    async fn get_amount_delegate(
        &self,
        delegator: Delegator,
        staker: Staker,
        epoch: Epoch,
    ) -> Result<Option<Amount>, Error> {
        let mut prefix = SmtPrefixType::Epoch(epoch).as_prefix();
        prefix.append(&mut SmtPrefixType::Address(staker).as_prefix());

        let snapshot = self.db.snapshot();
        let smt = get_smt!(self.db, &DELEGATOR_TABLE, &prefix, &snapshot);

        let leaf_value = smt.get(&SmtKeyEncode::Address(delegator).to_h256())?;
        if leaf_value == LeafValue::zero() {
            return Ok(None);
        }

        Ok(Some(Amount::from(leaf_value)))
    }

    async fn get_sub_leaves_delegate(
        &self,
        staker: Staker,
        epoch: Epoch,
    ) -> Result<HashMap<Delegator, Amount>, Error> {
        let mut prefix = SmtPrefixType::Epoch(epoch).as_prefix();
        prefix.append(&mut SmtPrefixType::Address(staker).as_prefix());

        self.get_sub_leaves(&prefix, &DELEGATOR_TABLE).await
    }

    async fn get_sub_root_delegate(
        &self,
        staker: Staker,
        epoch: Epoch,
    ) -> Result<Option<Root>, Error> {
        let mut prefix = SmtPrefixType::Epoch(epoch).as_prefix();
        prefix.append(&mut SmtPrefixType::Address(staker).as_prefix());

        let snapshot = self.db.snapshot();
        let smt = get_smt!(self.db, &DELEGATOR_TABLE, &prefix, &snapshot);

        Ok(Some(smt.root().clone()))
    }

    async fn get_sub_roots_delegate(
        &self,
        staker: Staker,
        epochs: Vec<Epoch>,
    ) -> Result<HashMap<Epoch, Option<Root>>, Error> {
        let mut hash_map = HashMap::new();

        for epoch in epochs {
            let root = self.get_sub_root_delegate(staker, epoch).await?;
            hash_map.insert(epoch, root);
        }

        Ok(hash_map)
    }

    async fn get_top_root_delegate(&self, staker: Staker) -> Result<Root, Error> {
        let mut prefix = SmtPrefixType::Top.as_prefix();
        prefix.append(&mut SmtPrefixType::Address(staker).as_prefix());
        let snapshot = self.db.snapshot();
        let smt = get_smt!(self.db, &DELEGATOR_TABLE, &prefix, &snapshot);

        Ok(smt.root().clone())
    }

    async fn get_top_roots_delegate(
        &self,
        stakers: Vec<Staker>,
    ) -> Result<HashMap<Staker, Root>, Error> {
        let mut hash_map = HashMap::new();
        for staker in stakers {
            let root = self.get_top_root_delegate(staker).await?;
            hash_map.insert(staker, root);
        }

        Ok(hash_map)
    }

    async fn generate_sub_proof_delegate(
        &self,
        staker: Staker,
        epoch: Epoch,
        delegators: Vec<Delegator>,
    ) -> Result<Proof, Error> {
        let mut prefix = SmtPrefixType::Epoch(epoch).as_prefix();
        prefix.append(&mut SmtPrefixType::Address(staker).as_prefix());

        let snapshot = self.db.snapshot();
        let mut keys = Vec::new();
        for validator in delegators {
            keys.push(SmtKeyEncode::Address(validator).to_h256());
        }

        let smt = get_smt!(self.db, &DELEGATOR_TABLE, &prefix, &snapshot);

        Ok(smt.merkle_proof(keys.clone())?.compile(keys)?.into())
    }

    async fn generate_top_proof_delegate(
        &self,
        staker: Staker,
        epochs: Vec<Epoch>,
    ) -> Result<Proof, Error> {
        let mut prefix = SmtPrefixType::Top.as_prefix();
        prefix.append(&mut SmtPrefixType::Address(staker).as_prefix());

        let snapshot = self.db.snapshot();
        let mut keys = Vec::new();
        for epoch in epochs {
            keys.push(SmtKeyEncode::Epoch(epoch).to_h256());
        }

        let smt = get_smt!(self.db, &DELEGATOR_TABLE, &prefix, &snapshot);

        Ok(smt.merkle_proof(keys.clone())?.compile(keys)?.into())
    }
}

#[async_trait]
impl RewardSmtStorage for SmtManager {
    async fn insert_reward(&self, address: Address, epoch: Epoch) -> Result<(), Error> {
        let kvs = vec![(
            SmtKeyEncode::Address(address).to_h256(),
            SmtValueEncode::Epoch(epoch).to_leaf_value(),
        )];

        let inner = self.db.transaction_default();
        let mut smt = get_smt!(self.db, &REWARD_TABLE, &inner);
        smt.update_all(kvs).expect("update_all error");
        inner.commit().expect("db commit error");
        Ok(())
    }

    async fn get_root_reward(&self) -> Result<Root, Error> {
        let snapshot = self.db.snapshot();
        let smt = get_smt!(self.db, &REWARD_TABLE, &snapshot);

        Ok(smt.root().clone())
    }

    async fn get_epoch_reward(&self, address: Address) -> Result<Option<Epoch>, Error> {
        let snapshot = self.db.snapshot();
        let smt = get_smt!(self.db, &REWARD_TABLE, &snapshot);

        let leaf_value = smt.get(&SmtKeyEncode::Address(address).to_h256())?;
        if leaf_value == LeafValue::zero() {
            return Ok(None);
        }

        Ok(Some(Epoch::from(leaf_value)))
    }

    async fn generate_proof_reward(&self, addresses: Vec<Address>) -> Result<Proof, Error> {
        let snapshot = self.db.snapshot();
        let smt = get_smt!(self.db, &REWARD_TABLE, &snapshot);

        let mut keys = Vec::new();
        for address in addresses {
            keys.push(SmtKeyEncode::Address(address).to_h256());
        }

        Ok(smt.merkle_proof(keys.clone())?.compile(keys)?.into())
    }
}

#[async_trait]
impl ProposalSmtStorage for SmtManager {
    async fn insert_proposal(
        &self,
        epoch: Epoch,
        proposals: Vec<(Validator, ProposalCount)>,
    ) -> Result<(), Error> {
        let kvs = proposals
            .into_iter()
            .map(|(k, v)| {
                (
                    SmtKeyEncode::Address(k).to_h256(),
                    SmtValueEncode::ProposalCount(v).to_leaf_value(),
                )
            })
            .collect();

        self.update(
            &PROPOSAL_TABLE,
            &SmtPrefixType::Epoch(epoch).as_prefix(),
            kvs,
        )?;

        let root = self.get_sub_root_proposal(epoch).await?.unwrap();
        let top_kvs = vec![(
            SmtKeyEncode::Epoch(epoch).to_h256(),
            SmtValueEncode::Root(root).to_leaf_value(),
        )];

        self.update(&PROPOSAL_TABLE, &SmtPrefixType::Top.as_prefix(), top_kvs)
    }

    async fn get_count_proposal(
        &self,
        epoch: Epoch,
    ) -> Result<HashMap<Validator, ProposalCount>, Error> {
        let mut hash_map = HashMap::new();

        let prefix = SmtPrefixType::Epoch(epoch).as_prefix();
        let prefix_len = prefix.len();
        let key_len = prefix_len + 32;
        let mode = IteratorMode::From(&prefix, Direction::Forward);
        let read_opt = ReadOptions::default();
        let cf = self
            .db
            .cf_handle(&format!("{}_{}", PROPOSAL_TABLE.to_string(), "leaf"))
            .unwrap();
        let cf_iter = self.db.get_iter_cf(cf, &read_opt, mode).unwrap();
        let kvs: Vec<(Validator, ProposalCount)> = cf_iter
            .into_iter()
            .take_while(|(k, _)| k.starts_with(&prefix))
            .filter_map(|(k, v)| {
                if key_len != key_len {
                    None
                } else {
                    let leaf_key: [u8; 32] = k[prefix_len..].try_into().expect("checked 32 bytes");
                    let leaf_value: [u8; 32] = v[..].try_into().expect("checked 32 bytes");
                    Some((
                        Validator::from_slice(&SmtKeyDecode::Address(leaf_key).from_h256()),
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

    async fn get_sub_root_proposal(&self, epoch: Epoch) -> Result<Option<Root>, Error> {
        let prefix = SmtPrefixType::Epoch(epoch).as_prefix();
        let snapshot = self.db.snapshot();
        let smt = get_smt!(self.db, &PROPOSAL_TABLE, &prefix, &snapshot);

        Ok(Some(smt.root().clone()))
    }

    async fn get_sub_roots_proposal(
        &self,
        epochs: Vec<Epoch>,
    ) -> Result<HashMap<Epoch, Option<Root>>, Error> {
        let mut hash_map = HashMap::new();

        for epoch in epochs {
            let root = self.get_sub_root_proposal(epoch).await?;
            hash_map.insert(epoch, root);
        }

        Ok(hash_map)
    }

    async fn get_top_root_proposal(&self) -> Result<Root, Error> {
        let prefix = SmtPrefixType::Top.as_prefix();
        let snapshot = self.db.snapshot();
        let smt = get_smt!(self.db, &PROPOSAL_TABLE, &prefix, &snapshot);
        Ok(smt.root().clone())
    }

    async fn generate_sub_proof_proposal(
        &self,
        epoch: Epoch,
        validators: Vec<Validator>,
    ) -> Result<Proof, Error> {
        let prefix = SmtPrefixType::Epoch(epoch).as_prefix();
        let snapshot = self.db.snapshot();
        let mut keys = Vec::new();
        for validator in validators {
            keys.push(SmtKeyEncode::Address(validator).to_h256());
        }

        let smt = get_smt!(self.db, &PROPOSAL_TABLE, &prefix, &snapshot);

        Ok(smt.merkle_proof(keys.clone())?.compile(keys)?.into())
    }

    async fn generate_top_proof_proposal(&self, epochs: Vec<Epoch>) -> Result<Proof, Error> {
        let prefix = SmtPrefixType::Top.as_prefix();
        let snapshot = self.db.snapshot();
        let mut keys = Vec::new();
        for epoch in epochs {
            keys.push(SmtKeyEncode::Epoch(epoch).to_h256());
        }

        let smt = get_smt!(self.db, &PROPOSAL_TABLE, &prefix, &snapshot);

        Ok(smt.merkle_proof(keys.clone())?.compile(keys)?.into())
    }
}
