use crate::types::H160;

use derive_more::Display;
use rocksdb::DBVector;
use sparse_merkle_tree::{traits::Value, H256};

lazy_static::lazy_static! {
    pub static ref TOP_SMT_PREFIX: &'static str = "top_smt";
    pub static ref STAKER_TABLE: &'static str = "staker";
    pub static ref DELEGATOR_TABLE: &'static str = "delegator";
    pub static ref REWARD_TABLE: &'static str = "reward";
    pub static ref PROPOSAL_TABLE: &'static str = "proposal";
}

pub type Amount = u128;
pub type Epoch = u64;
pub type Proof = Vec<u8>;
pub type ProposalCount = u64;
pub type Root = H256;

pub type Address = H160;
pub type Staker = H160;
pub type Delegator = H160;
pub type Validator = H160;

// todo: refactor, is_increase no longer needed
#[derive(Clone, Debug)]
pub struct UserAmount {
    pub user:        Address,
    pub amount:      Amount,
    pub is_increase: bool,
}

#[derive(Clone, Debug, Display)]
pub enum CFSuffixType {
    #[display(fmt = "branch")]
    Branch,
    #[display(fmt = "leaf")]
    Leaf,
}

pub enum SmtPrefixType {
    Top,
    Epoch(Epoch),
    Address(Address),
}

impl SmtPrefixType {
    pub fn as_prefix(&self) -> Vec<u8> {
        // Encode different types into SMT prefix type Vec<u8>
        match self {
            SmtPrefixType::Top => TOP_SMT_PREFIX.as_bytes().to_vec(),
            SmtPrefixType::Epoch(epoch) => epoch.to_le_bytes().to_vec(),
            SmtPrefixType::Address(address) => address.to_fixed_bytes().to_vec(),
        }
    }
}

pub enum SmtKeyEncode {
    Epoch(Epoch),
    Address(Address),
}

impl SmtKeyEncode {
    pub fn to_h256(&self) -> H256 {
        // Encode different types into SMT key type H256
        match self {
            SmtKeyEncode::Epoch(epoch) => {
                let mut buf = [0u8; 32];
                buf[..8].copy_from_slice(&epoch.to_le_bytes());
                buf.into()
            }
            SmtKeyEncode::Address(address) => {
                let mut buf = [0u8; 32];
                buf[..20].copy_from_slice(&address.to_fixed_bytes());
                buf.into()
            }
        }
    }
}

pub enum SmtValueEncode {
    Amount(Amount),
    Epoch(Epoch),
    ProposalCount(ProposalCount),
    Root(Root),
}

impl SmtValueEncode {
    pub fn to_leaf_value(&self) -> LeafValue {
        // Encode different type to LeafValue
        match self {
            SmtValueEncode::Amount(amount) => (*amount).into(),
            SmtValueEncode::Epoch(epoch) => (*epoch).into(),
            SmtValueEncode::ProposalCount(count) => (*count).into(),
            SmtValueEncode::Root(root) => LeafValue::from(*root),
        }
    }
}

// Define SMT value
#[derive(Default, Clone, PartialEq, Eq, Debug)]
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

impl AsRef<[u8]> for LeafValue {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

/// LeafValue <-> Amount
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

/// LeafValue <-> u64 (Epoch or ProposalCount)
impl From<u64> for LeafValue {
    fn from(v: u64) -> Self {
        let count_bytes = v.to_le_bytes();
        let mut buf = [0u8; 32];
        buf[..8].copy_from_slice(&count_bytes);
        LeafValue(buf)
    }
}

impl From<LeafValue> for u64 {
    fn from(leaf_value: LeafValue) -> Self {
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&leaf_value.0[..8]);
        u64::from_le_bytes(buf)
    }
}

/// LeafValue <-> Root
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
