use std::{collections::HashMap, path::PathBuf, sync::Arc, vec};

use anyhow::Result;
use async_trait::async_trait;

use rocksdb::{prelude::*, Direction, IteratorMode, OptimisticTransactionDB};
use smt_rocksdb_store::cf_store::{ColumnFamilyStore, ColumnFamilyStoreMultiTree};
use sparse_merkle_tree::{traits::Value, H256};

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

/// SMT manager
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

    async fn get_sub_leaves(&self, prefix: &[u8], table: &str) -> Result<HashMap<Address, Amount>> {
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

    fn update(&self, cf: &str, prefix: &[u8], kvs: Vec<(H256, LeafValue)>) -> Result<()> {
        let inner = self.db.transaction_default();
        let mut smt = get_smt!(self.db, cf, prefix, &inner);
        smt.update_all(kvs).expect("update_all error");
        inner.commit()?;
        Ok(())
    }
}

/// Staker SMT
/// For sub smt, the key is the staker address, the value is the amount of
/// staking. For top smt, the key is the epoch, the value is the root of the sub
/// smt.                          
///                          Staker Root
///                /                             \
///          epoch 1 root                   epoch 2 root
///         /      |       \               /      |        \
///    staker1  staker2  staker3       staker1  staker3  staker4
///    amount1  amount2  amount3       amount1  amount3  amount4
///
/// Column family prefix in RocksDB: "staker" --> "staker_branch" and
/// "staker_leaf" Tree prefix in Column family: epoch
///
/// Top SMT
///     key: epoch(u64).to_le_bytes() + [0u8; 24]
///     value: root(H256)
///
/// Sub SMT
///     key: staker_address(H160).to_fixed_bytes() + [0u8; 12]
///     value: amount(u128).to_fixed_bytes() + [0u8; 16]
#[async_trait]
impl StakeSmtStorage for SmtManager {
    async fn insert(&self, epoch: Epoch, amounts: Vec<(Address, Amount)>) -> Result<()> {
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

        let root = StakeSmtStorage::get_sub_root(self, epoch).await?.unwrap();
        let top_kvs = vec![(
            SmtKeyEncode::Epoch(epoch).to_h256(),
            SmtValueEncode::Root(root).to_leaf_value(),
        )];

        self.update(&STAKER_TABLE, &SmtPrefixType::Top.as_prefix(), top_kvs)
    }

    async fn remove(&self, epoch: Epoch, address: Address) -> Result<()> {
        let kvs = vec![(SmtKeyEncode::Address(address).to_h256(), LeafValue::zero())];

        self.update(&STAKER_TABLE, &SmtPrefixType::Epoch(epoch).as_prefix(), kvs)?;

        let root = StakeSmtStorage::get_sub_root(self, epoch).await?.unwrap();
        let top_kvs = vec![(
            SmtKeyEncode::Epoch(epoch).to_h256(),
            SmtValueEncode::Root(root).to_leaf_value(),
        )];

        self.update(&STAKER_TABLE, &SmtPrefixType::Top.as_prefix(), top_kvs)
    }

    async fn remove_batch(&self, epoch: Epoch, addresses: Vec<Address>) -> Result<()> {
        let kvs = addresses
            .into_iter()
            .map(|k| (SmtKeyEncode::Address(k).to_h256(), LeafValue::zero()))
            .collect();

        self.update(&STAKER_TABLE, &SmtPrefixType::Epoch(epoch).as_prefix(), kvs)?;

        let root = StakeSmtStorage::get_sub_root(self, epoch).await?.unwrap();
        let top_kvs = vec![(
            SmtKeyEncode::Epoch(epoch).to_h256(),
            SmtValueEncode::Root(root).to_leaf_value(),
        )];

        self.update(&STAKER_TABLE, &SmtPrefixType::Top.as_prefix(), top_kvs)
    }

    async fn get_amount(&self, epoch: Epoch, address: Address) -> Result<Option<Amount>> {
        let prefix = SmtPrefixType::Epoch(epoch).as_prefix();
        let snapshot = self.db.snapshot();
        let smt = get_smt!(self.db, &STAKER_TABLE, &prefix, &snapshot);

        let leaf_value = smt.get(&SmtKeyEncode::Address(address).to_h256())?;
        if leaf_value == LeafValue::zero() {
            return Ok(None);
        }

        Ok(Some(Amount::from(leaf_value)))
    }

    async fn get_sub_leaves(&self, epoch: Epoch) -> Result<HashMap<Address, Amount>> {
        let prefix = SmtPrefixType::Epoch(epoch).as_prefix();
        self.get_sub_leaves(&prefix, &STAKER_TABLE).await
    }

    async fn get_sub_root(&self, epoch: Epoch) -> Result<Option<Root>> {
        let prefix = SmtPrefixType::Epoch(epoch).as_prefix();
        let snapshot = self.db.snapshot();
        let smt = get_smt!(self.db, &STAKER_TABLE, &prefix, &snapshot);

        Ok(Some(smt.root().clone()))
    }

    async fn get_sub_roots(&self, epochs: Vec<Epoch>) -> Result<HashMap<Epoch, Option<Root>>> {
        let mut hash_map = HashMap::new();

        for epoch in epochs {
            let root = StakeSmtStorage::get_sub_root(self, epoch).await?;
            hash_map.insert(epoch, root);
        }

        Ok(hash_map)
    }

    async fn get_top_root(&self) -> Result<Root> {
        let prefix = SmtPrefixType::Top.as_prefix();
        let snapshot = self.db.snapshot();
        let smt = get_smt!(self.db, &STAKER_TABLE, &prefix, &snapshot);

        Ok(smt.root().clone())
    }

    async fn generate_sub_proof(&self, epoch: Epoch, addresses: Vec<Address>) -> Result<Proof> {
        let prefix = SmtPrefixType::Epoch(epoch).as_prefix();
        let snapshot = self.db.snapshot();
        let keys = addresses
            .into_iter()
            .map(|k| SmtKeyEncode::Address(k).to_h256())
            .collect::<Vec<H256>>();
        let smt = get_smt!(self.db, &STAKER_TABLE, &prefix, &snapshot);

        Ok(smt.merkle_proof(keys.clone())?.compile(keys)?.into())
    }

    async fn generate_top_proof(&self, epochs: Vec<Epoch>) -> Result<Proof> {
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

/// Delegator SMTs
/// Each smt stores one staker's delegation information.
/// For sub smt, the key is the delegator address, the value is the amount of
/// delegation. For top smt, the key is the epoch, the value is the root of sub
/// smt.      
///                               Staker 1 Root
///                  /                                     \
///            epoch 1 root       ...                 epoch 2 root
///      /          |           \               /          |          \
/// delegator1  delegator2  delegator3      delegator1  delegator2  delegator4
///  amount1     amount2     amount3        amount1     amount2     amount4
///                                     .
///                                     .
///                                     .
///                               Staker k Root
///                  /                                     \
///            epoch 1 root       ...                 epoch 2 root
///      /          |            \              /          |          \
/// delegator1  delegator2  delegator3      delegator1  delegator2  delegator4
///  amount1     amount2     amount3         amount1     amount2     amount4
///
///  Column family prefix in RocksDB: 'delegator' --> "delegator_branch" and
///  "delegator_leaf" Tree prefix in Column family: epoch.to_le_bytes() +
///  stake_address.to_fixed_bytes()
///
///  Top SMT
///      key: 'top_smt'.as_slice() + staker_address.to_fixed_bytes()
///      value: root(H256)
///
///  Sub SMT
///     key: delegator_address(H160).to_fixed_bytes() + [0u8; 12]
///     value: amount(u128).to_fixed_bytes() + [0u8; 16]
#[async_trait]
impl DelegateSmtStorage for SmtManager {
    async fn insert(
        &self,
        epoch: Epoch,
        delegators: HashMap<Staker, Vec<(Delegator, Amount)>>,
    ) -> Result<()> {
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

            let root = DelegateSmtStorage::get_sub_root(self, staker, epoch)
                .await?
                .unwrap();
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

    async fn remove(
        &self,
        epoch: Epoch,
        delegators: HashMap<Staker, Vec<Delegator>>,
    ) -> Result<()> {
        let mut hash_map = HashMap::new();
        for (staker, addresses) in delegators {
            let kvs = addresses
                .into_iter()
                .map(|k| (k, Amount::default()))
                .collect();
            hash_map.insert(staker, kvs);
        }

        DelegateSmtStorage::insert(self, epoch, hash_map).await
    }

    async fn get_amount(
        &self,
        delegator: Delegator,
        staker: Staker,
        epoch: Epoch,
    ) -> Result<Option<Amount>> {
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

    async fn get_sub_leaves(
        &self,
        staker: Staker,
        epoch: Epoch,
    ) -> Result<HashMap<Delegator, Amount>> {
        let mut prefix = SmtPrefixType::Epoch(epoch).as_prefix();
        prefix.append(&mut SmtPrefixType::Address(staker).as_prefix());

        self.get_sub_leaves(&prefix, &DELEGATOR_TABLE).await
    }

    async fn get_sub_root(&self, staker: Staker, epoch: Epoch) -> Result<Option<Root>> {
        let mut prefix = SmtPrefixType::Epoch(epoch).as_prefix();
        prefix.append(&mut SmtPrefixType::Address(staker).as_prefix());

        let snapshot = self.db.snapshot();
        let smt = get_smt!(self.db, &DELEGATOR_TABLE, &prefix, &snapshot);

        Ok(Some(smt.root().clone()))
    }

    async fn get_sub_roots(
        &self,
        staker: Staker,
        epochs: Vec<Epoch>,
    ) -> Result<HashMap<Epoch, Option<Root>>> {
        let mut hash_map = HashMap::new();

        for epoch in epochs {
            let root = DelegateSmtStorage::get_sub_root(self, staker, epoch).await?;
            hash_map.insert(epoch, root);
        }

        Ok(hash_map)
    }

    async fn get_top_root(&self, staker: Staker) -> Result<Root> {
        let mut prefix = SmtPrefixType::Top.as_prefix();
        prefix.append(&mut SmtPrefixType::Address(staker).as_prefix());
        let snapshot = self.db.snapshot();
        let smt = get_smt!(self.db, &DELEGATOR_TABLE, &prefix, &snapshot);

        Ok(smt.root().clone())
    }

    async fn get_top_roots(&self, stakers: Vec<Staker>) -> Result<HashMap<Staker, Root>> {
        let mut hash_map = HashMap::new();
        for staker in stakers {
            let root = DelegateSmtStorage::get_top_root(self, staker).await?;
            hash_map.insert(staker, root);
        }

        Ok(hash_map)
    }

    async fn generate_sub_proof(
        &self,
        staker: Staker,
        epoch: Epoch,
        delegators: Vec<Delegator>,
    ) -> Result<Proof> {
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

    async fn generate_top_proof(&self, staker: Staker, epochs: Vec<Epoch>) -> Result<Proof> {
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

/// Reward SMT
/// For the smt, the key is the address, the value is the last epoch that the
/// reward has been claimed.
///               Reward Root
///        /         |          \
///    address1   address2   address3
///     epoch1     epoch2     epoch3
///
/// Column family prefix in RocksDB: 'reward' --> "reward_branch" and
/// "reward_leaf" There is only a single tree in the column family.
///
/// SMT
///    key: address(H160).to_fixed_bytes() + [0u8; 12]
///    value: epoch(u64).to_le_bytes() + [0u8; 24]
#[async_trait]
impl RewardSmtStorage for SmtManager {
    async fn insert(&self, address: Address, epoch: Epoch) -> Result<()> {
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

    async fn get_root(&self) -> Result<Root> {
        let snapshot = self.db.snapshot();
        let smt = get_smt!(self.db, &REWARD_TABLE, &snapshot);

        Ok(smt.root().clone())
    }

    async fn get_epoch(&self, address: Address) -> Result<Option<Epoch>> {
        let snapshot = self.db.snapshot();
        let smt = get_smt!(self.db, &REWARD_TABLE, &snapshot);

        let leaf_value = smt.get(&SmtKeyEncode::Address(address).to_h256())?;
        if leaf_value == LeafValue::zero() {
            return Ok(None);
        }

        Ok(Some(Epoch::from(leaf_value)))
    }

    async fn generate_proof(&self, addresses: Vec<Address>) -> Result<Proof> {
        let snapshot = self.db.snapshot();
        let smt = get_smt!(self.db, &REWARD_TABLE, &snapshot);

        let mut keys = Vec::new();
        for address in addresses {
            keys.push(SmtKeyEncode::Address(address).to_h256());
        }

        Ok(smt.merkle_proof(keys.clone())?.compile(keys)?.into())
    }
}

/// Proposal SMT
/// For sub smt, the key is the validator address, the value is the amount of
/// proposals. For top smt, the key is the epoch, the value is the root of sub
/// smt.  
///                                Proposal Root
///                   /                                    \
///             epoch 1 root                           epoch 2 root
///      /           |           \              /           |            \
/// validator1   validator2   validator3    validator1   validator2   validator4
///   count1       count2       count3        count1       count2       count4
///
/// Column family prefix in RocksDB: "proposal" --> "proposal_branch" and
/// "proposal_leaf" Tree prefix in Column family: epoch
///
/// Top SMT
///     key: epoch(u64).to_le_bytes() + [0u8; 24]
///     value: root(H256)
///
/// Sub SMT
///     key: validator_address(H160).to_fixed_bytes() + [0u8; 12]
///     value: count(u64).to_fixed_bytes() + [0u8; 24]
#[async_trait]
impl ProposalSmtStorage for SmtManager {
    async fn insert(&self, epoch: Epoch, proposals: Vec<(Validator, ProposalCount)>) -> Result<()> {
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

        let root = ProposalSmtStorage::get_sub_root(self, epoch)
            .await?
            .unwrap();
        let top_kvs = vec![(
            SmtKeyEncode::Epoch(epoch).to_h256(),
            SmtValueEncode::Root(root).to_leaf_value(),
        )];

        self.update(&PROPOSAL_TABLE, &SmtPrefixType::Top.as_prefix(), top_kvs)
    }

    async fn get_count(&self, epoch: Epoch) -> Result<HashMap<Validator, ProposalCount>> {
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

    async fn get_sub_root(&self, epoch: Epoch) -> Result<Option<Root>> {
        let prefix = SmtPrefixType::Epoch(epoch).as_prefix();
        let snapshot = self.db.snapshot();
        let smt = get_smt!(self.db, &PROPOSAL_TABLE, &prefix, &snapshot);

        Ok(Some(smt.root().clone()))
    }

    async fn get_sub_roots(&self, epochs: Vec<Epoch>) -> Result<HashMap<Epoch, Option<Root>>> {
        let mut hash_map = HashMap::new();

        for epoch in epochs {
            let root = ProposalSmtStorage::get_sub_root(self, epoch).await?;
            hash_map.insert(epoch, root);
        }

        Ok(hash_map)
    }

    async fn get_top_root(&self) -> Result<Root> {
        let prefix = SmtPrefixType::Top.as_prefix();
        let snapshot = self.db.snapshot();
        let smt = get_smt!(self.db, &PROPOSAL_TABLE, &prefix, &snapshot);
        Ok(smt.root().clone())
    }

    async fn generate_sub_proof(&self, epoch: Epoch, validators: Vec<Validator>) -> Result<Proof> {
        let prefix = SmtPrefixType::Epoch(epoch).as_prefix();
        let snapshot = self.db.snapshot();
        let mut keys = Vec::new();
        for validator in validators {
            keys.push(SmtKeyEncode::Address(validator).to_h256());
        }

        let smt = get_smt!(self.db, &PROPOSAL_TABLE, &prefix, &snapshot);

        Ok(smt.merkle_proof(keys.clone())?.compile(keys)?.into())
    }

    async fn generate_top_proof(&self, epochs: Vec<Epoch>) -> Result<Proof> {
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
