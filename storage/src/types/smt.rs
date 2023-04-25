use ethereum_types::H160;
use rocksdb::DBVector;
use smt_rocksdb_store::default_store::{DefaultStore, DefaultStoreMultiTree};
use sparse_merkle_tree::{blake2b::Blake2bHasher, traits::Value, SparseMerkleTree, H256};

pub type DefaultStoreSMT<'a, T, W> =
    SparseMerkleTree<Blake2bHasher, LeafValue, DefaultStore<'a, T, W>>;

pub type DefaultStoreMultiSMT<'a, T, W> =
    SparseMerkleTree<Blake2bHasher, LeafValue, DefaultStoreMultiTree<'a, T, W>>;

pub type Amount = u128;
pub type Epoch = u64;
pub type Proof = Vec<u8>;
pub type ProposalCount = u64;
pub type Root = H256;

pub type Address = H160;
pub type Staker = H160;
pub type Delegator = H160;
pub type Validator = H160;

// define SMT value
#[derive(Default, Clone, PartialEq, Eq)]
pub struct LeafValue(pub [u8; 32]);
impl Value for LeafValue {
    fn to_h256(&self) -> H256 {
        self.0.into()
    }

    fn zero() -> Self {
        Self([0u8; 32])
    }
}

impl From<DBVector> for LeafValue {
    fn from(vec: DBVector) -> Self {
        LeafValue(vec.as_ref().try_into().expect("stored value is 32 bytes"))
    }
}

impl From<Amount> for LeafValue {
    fn from(amount: Amount) -> Self {
        let amount_bytes = amount.to_le_bytes();
        let mut buf = [0u8; 32];
        buf[..16].copy_from_slice(&amount_bytes);
        LeafValue(buf)
    }
}

impl From<LeafValue> for Amount {
    fn from(leaf_value: LeafValue) -> Self {
        let mut buf = [0u8; 16];
        buf.copy_from_slice(&leaf_value.0[..16]);
        Amount::from_le_bytes(buf)
    }
}

impl From<ProposalCount> for LeafValue {
    fn from(count: ProposalCount) -> Self {
        let count_bytes = count.to_le_bytes();
        let mut buf = [0u8; 32];
        buf[..8].copy_from_slice(&count_bytes);
        LeafValue(buf)
    }
}

impl From<LeafValue> for ProposalCount {
    fn from(leaf_value: LeafValue) -> Self {
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&leaf_value.0[..8]);
        ProposalCount::from_le_bytes(buf)
    }
}

impl From<Root> for LeafValue {
    fn from(root: Root) -> Self {
        LeafValue(<[u8; 32]>::from(root))
    }
}

impl From<LeafValue> for Root {
    fn from(leaf_value: LeafValue) -> Self {
        leaf_value.0.into()
    }
}

impl AsRef<[u8]> for LeafValue {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}
