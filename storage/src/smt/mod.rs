mod utils;

use std::{collections::HashMap, fs, path::PathBuf, sync::Arc, vec};

use anyhow::Result;
use async_trait::async_trait;

use rocksdb::{prelude::*, Direction, IteratorMode, OptimisticTransactionDB};
use smt_rocksdb_store::cf_store::{ColumnFamilyStore, ColumnFamilyStoreMultiTree};
use sparse_merkle_tree::{traits::Value, H256};

use common::{
    traits::smt::{DelegateSmtStorage, ProposalSmtStorage, RewardSmtStorage, StakeSmtStorage},
    types::smt::{
        Address, Amount, CFSuffixType, ColumnFamilyStoreMultiSMT, ColumnFamilyStoreSMT, Delegator,
        Epoch, LeafValue, Proof, ProposalCount, Root, SmtKeyEncode, SmtPrefixType, SmtValueEncode,
        Staker, UserAmount, Validator, DELEGATOR_TABLE, PROPOSAL_TABLE, REWARD_TABLE, STAKER_TABLE,
    },
};

use crate::error::StorageError;
use crate::{create_table_cfs, get_cf_prefix, get_smt, get_sub_leaves, keys_to_h256};

pub struct SmtManager {
    db: Arc<OptimisticTransactionDB>,
}

/// SMT manager
impl SmtManager {
    pub fn new(path: PathBuf) -> Self {
        if !path.is_dir() {
            fs::create_dir_all(&path)
                .map_err(StorageError::RocksDBCreationError)
                .unwrap();
        }

        let mut db_opts = Options::default();
        db_opts.create_missing_column_families(true);
        db_opts.create_if_missing(true);

        let mut cfs = vec![];
        cfs.extend_from_slice(create_table_cfs!(STAKER_TABLE));
        cfs.extend_from_slice(create_table_cfs!(DELEGATOR_TABLE));
        cfs.extend_from_slice(create_table_cfs!(REWARD_TABLE));
        cfs.extend_from_slice(create_table_cfs!(PROPOSAL_TABLE));

        let db = OptimisticTransactionDB::open_cf(&db_opts, path, cfs).unwrap();

        Self { db: Arc::new(db) }
    }

    async fn insert_full(
        &self,
        epoch: Epoch,
        delegators: HashMap<Staker, Vec<(H256, LeafValue)>>,
    ) -> Result<()> {
        for (staker, amounts) in delegators {
            let current_prefix = get_cf_prefix!(Epoch, epoch, Address, staker);
            self.update(&DELEGATOR_TABLE, &current_prefix, amounts)?;

            let root = DelegateSmtStorage::get_sub_root(self, staker, epoch)
                .await?
                .unwrap();
            let top_kvs = vec![(
                SmtKeyEncode::Epoch(epoch).to_h256(),
                SmtValueEncode::Root(root).to_leaf_value(),
            )];

            let top_prefix = get_cf_prefix!(Address, staker);
            self.update(&DELEGATOR_TABLE, &top_prefix, top_kvs)?;
        }
        Ok(())
    }

    fn update(&self, cf: &str, prefix: &[u8], kvs: Vec<(H256, LeafValue)>) -> Result<()> {
        let inner = self.db.transaction_default();
        let mut smt = get_smt!(self.db, cf, prefix, &inner);
        smt.update_all(kvs)?;
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
    async fn insert(&self, epoch: Epoch, amounts: Vec<UserAmount>) -> Result<()> {
        // aggregate staking records
        let hash_map = amounts
            .into_iter()
            .fold(HashMap::new(), |mut hash_map, record| {
                let UserAmount {
                    user,
                    amount,
                    is_increase,
                } = record;
                hash_map
                    .entry(user)
                    .or_insert_with(Vec::new)
                    .push((amount, is_increase));
                hash_map
            });

        let mut kvs = vec![];
        for (addr, amounts) in hash_map {
            let mut stored_amount = StakeSmtStorage::get_amount(self, epoch, addr)
                .await?
                .unwrap_or_default();
            let _ = amounts
                .into_iter()
                .map(|(v, inc)| {
                    stored_amount = if inc {
                        stored_amount + v
                    } else {
                        stored_amount.saturating_sub(v)
                    };
                })
                .collect::<()>();
            kvs.push((
                SmtKeyEncode::Address(addr).to_h256(),
                SmtValueEncode::Amount(stored_amount).to_leaf_value(),
            ));
        }

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

        Ok(get_sub_leaves!(
            Amount,
            &prefix,
            self.db,
            STAKER_TABLE.to_string()
        ))
    }

    async fn get_sub_root(&self, epoch: Epoch) -> Result<Option<Root>> {
        let prefix = SmtPrefixType::Epoch(epoch).as_prefix();
        let snapshot = self.db.snapshot();
        let smt = get_smt!(self.db, &STAKER_TABLE, &prefix, &snapshot);

        Ok(Some(smt.root().clone()))
    }

    async fn get_sub_roots(&self, epochs: Vec<Epoch>) -> Result<HashMap<Epoch, Option<Root>>> {
        let mut hash_map = HashMap::with_capacity(epochs.len());

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
        let keys = keys_to_h256!(addresses, Address);
        let smt = get_smt!(self.db, &STAKER_TABLE, &prefix, &snapshot);

        Ok(smt.merkle_proof(keys.clone())?.compile(keys)?.into())
    }

    async fn generate_top_proof(&self, epochs: Vec<Epoch>) -> Result<Proof> {
        let prefix = SmtPrefixType::Top.as_prefix();
        let snapshot = self.db.snapshot();
        let keys = keys_to_h256!(epochs, Epoch);
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
    async fn insert(&self, epoch: Epoch, delegators: Vec<(Staker, UserAmount)>) -> Result<()> {
        // aggregate records by staker
        let staker_hash_map =
            delegators
                .into_iter()
                .fold(HashMap::new(), |mut hash_map, record| {
                    let (staker, record) = record;
                    hash_map.entry(staker).or_insert_with(Vec::new).push(record);
                    hash_map
                });

        let mut updated_delegators = HashMap::with_capacity(staker_hash_map.len());
        for (staker, amounts) in staker_hash_map {
            // for each staker, aggregate records by delegator
            let delegator_hash_map =
                amounts
                    .into_iter()
                    .fold(HashMap::new(), |mut hash_map, record| {
                        let UserAmount {
                            user,
                            amount,
                            is_increase,
                        } = record;
                        hash_map
                            .entry(user)
                            .or_insert_with(Vec::new)
                            .push((amount, is_increase));
                        hash_map
                    });
            let mut kvs = vec![];
            for (delegator, amounts) in delegator_hash_map {
                let mut stored_amount =
                    DelegateSmtStorage::get_amount(self, delegator, staker, epoch)
                        .await?
                        .unwrap_or_default();
                let _ = amounts
                    .into_iter()
                    .map(|(v, inc)| {
                        stored_amount = if inc {
                            stored_amount + v
                        } else {
                            stored_amount.saturating_sub(v)
                        };
                    })
                    .collect::<()>();
                kvs.push((
                    SmtKeyEncode::Address(delegator).to_h256(),
                    SmtValueEncode::Amount(stored_amount).to_leaf_value(),
                ));
            }
            updated_delegators.insert(staker, kvs);
        }

        self.insert_full(epoch, updated_delegators).await
    }

    async fn remove(&self, epoch: Epoch, delegators: Vec<(Staker, Delegator)>) -> Result<()> {
        let removed_dalegators =
            delegators
                .into_iter()
                .fold(HashMap::new(), |mut hash_map, record| {
                    let (staker, delegator) = record;
                    hash_map.entry(staker).or_insert_with(Vec::new).push((
                        SmtKeyEncode::Address(delegator).to_h256(),
                        LeafValue::zero(),
                    ));
                    hash_map
                });

        self.insert_full(epoch, removed_dalegators).await
    }

    async fn get_amount(
        &self,
        delegator: Delegator,
        staker: Staker,
        epoch: Epoch,
    ) -> Result<Option<Amount>> {
        let prefix = get_cf_prefix!(Epoch, epoch, Address, staker);

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
        let prefix = get_cf_prefix!(Epoch, epoch, Address, staker);

        Ok(get_sub_leaves!(
            Amount,
            &prefix,
            self.db,
            DELEGATOR_TABLE.to_string()
        ))
    }

    async fn get_sub_root(&self, staker: Staker, epoch: Epoch) -> Result<Option<Root>> {
        let prefix = get_cf_prefix!(Epoch, epoch, Address, staker);

        let snapshot = self.db.snapshot();
        let smt = get_smt!(self.db, &DELEGATOR_TABLE, &prefix, &snapshot);

        Ok(Some(smt.root().clone()))
    }

    async fn get_sub_roots(
        &self,
        staker: Staker,
        epochs: Vec<Epoch>,
    ) -> Result<HashMap<Epoch, Option<Root>>> {
        let mut hash_map = HashMap::with_capacity(epochs.len());

        for epoch in epochs {
            let root = DelegateSmtStorage::get_sub_root(self, staker, epoch).await?;
            hash_map.insert(epoch, root);
        }

        Ok(hash_map)
    }

    async fn get_top_root(&self, staker: Staker) -> Result<Root> {
        let prefix = get_cf_prefix!(Address, staker);
        let snapshot = self.db.snapshot();
        let smt = get_smt!(self.db, &DELEGATOR_TABLE, &prefix, &snapshot);

        Ok(smt.root().clone())
    }

    async fn get_top_roots(&self, stakers: Vec<Staker>) -> Result<HashMap<Staker, Root>> {
        let mut hash_map = HashMap::with_capacity(stakers.len());
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
        let prefix = get_cf_prefix!(Epoch, epoch, Address, staker);

        let snapshot = self.db.snapshot();
        let keys = keys_to_h256!(delegators, Address);

        let smt = get_smt!(self.db, &DELEGATOR_TABLE, &prefix, &snapshot);

        Ok(smt.merkle_proof(keys.clone())?.compile(keys)?.into())
    }

    async fn generate_top_proof(&self, staker: Staker, epochs: Vec<Epoch>) -> Result<Proof> {
        let prefix = get_cf_prefix!(Address, staker);

        let snapshot = self.db.snapshot();
        let keys = keys_to_h256!(epochs, Epoch);
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
        smt.update_all(kvs)?;
        inner.commit()?;
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

        let mut keys = Vec::with_capacity(addresses.len());
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

    async fn get_count(&self, epoch: Epoch, validator: Address) -> Result<Option<ProposalCount>> {
        let prefix = SmtPrefixType::Epoch(epoch).as_prefix();
        let snapshot = self.db.snapshot();
        let smt = get_smt!(self.db, &PROPOSAL_TABLE, &prefix, &snapshot);

        let leaf_value = smt.get(&SmtKeyEncode::Address(validator).to_h256())?;
        if leaf_value == LeafValue::zero() {
            return Ok(None);
        }

        Ok(Some(ProposalCount::from(leaf_value)))
    }

    async fn get_sub_leaves(&self, epoch: Epoch) -> Result<HashMap<Validator, ProposalCount>> {
        let prefix = SmtPrefixType::Epoch(epoch).as_prefix();

        Ok(get_sub_leaves!(
            ProposalCount,
            &prefix,
            self.db,
            PROPOSAL_TABLE.to_string()
        ))
    }

    async fn get_sub_root(&self, epoch: Epoch) -> Result<Option<Root>> {
        let prefix = SmtPrefixType::Epoch(epoch).as_prefix();
        let snapshot = self.db.snapshot();
        let smt = get_smt!(self.db, &PROPOSAL_TABLE, &prefix, &snapshot);

        Ok(Some(smt.root().clone()))
    }

    async fn get_sub_roots(&self, epochs: Vec<Epoch>) -> Result<HashMap<Epoch, Option<Root>>> {
        let mut hash_map = HashMap::with_capacity(epochs.len());

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
        let keys = keys_to_h256!(validators, Address);

        let smt = get_smt!(self.db, &PROPOSAL_TABLE, &prefix, &snapshot);

        Ok(smt.merkle_proof(keys.clone())?.compile(keys)?.into())
    }

    async fn generate_top_proof(&self, epochs: Vec<Epoch>) -> Result<Proof> {
        let prefix = SmtPrefixType::Top.as_prefix();
        let snapshot = self.db.snapshot();
        let keys = keys_to_h256!(epochs, Epoch);

        let smt = get_smt!(self.db, &PROPOSAL_TABLE, &prefix, &snapshot);

        Ok(smt.merkle_proof(keys.clone())?.compile(keys)?.into())
    }
}
