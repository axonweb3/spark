use blake2b_rs::{Blake2b, Blake2bBuilder};
use ethereum_types::H160;
use sparse_merkle_tree::{
    blake2b::Blake2bHasher, default_store::DefaultStore, traits::Value, SparseMerkleTree, H256,
};

pub type SmtType = SparseMerkleTree<Blake2bHasher, LeafValue, DefaultStore<LeafValue>>;

pub type Address = H160;
pub type Amount = u128;
pub type Delegator = H160;
pub type Epoch = u64;
pub type Leaf = (H256, H256);
pub type Proof = Vec<u8>;
pub type Root = H256;
pub type Staker = H160;

// define SMT value
#[derive(Default, Clone, PartialEq, Eq)]
pub struct LeafValue(pub [u8; 32]);
impl Value for LeafValue {
    fn to_h256(&self) -> H256 {
        if self.0 == [0u8; 32] {
            return H256::zero();
        }
        let mut buf = [0u8; 32];
        let mut hasher = new_blake2b();
        hasher.update(&self.0);
        hasher.finalize(&mut buf);
        buf.into()
    }

    fn zero() -> Self {
        Default::default()
    }
}

// helper function
pub fn new_blake2b() -> Blake2b {
    Blake2bBuilder::new(32).personal(b"SMT").build()
}
