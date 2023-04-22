use blake2b_rs::{Blake2b, Blake2bBuilder};
use ethereum_types::{H160, U256};
use sparse_merkle_tree::{
    blake2b::Blake2bHasher, default_store::DefaultStore, traits::Value,
    SparseMerkleTree, H256,
};

pub type SmtType = SparseMerkleTree<Blake2bHasher, Amount, DefaultStore<Amount>>;
pub type Root = H256;
pub type Address = H160;
pub type Epoch = u64;
pub type Staker = H160;
pub type Delegator = H160;
pub type Proof = H256;
pub type Leaf = (H256, H256);

// define SMT value
#[derive(Default, Clone, PartialEq, Eq)]
pub struct Amount(U256);
impl Value for Amount {
    fn to_h256(&self) -> H256 {
        if self.0.is_zero() {
            return H256::zero();
        }
        let mut buf = [0u8; 32];
        let mut hasher = new_blake2b();
        hasher.update(u256_to_u8_slice(&self.0));
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

fn u256_to_u8_slice(index: &U256) -> &[u8] {
    let u64_slice = index.as_ref();
    let result: &[u8] = bytemuck::cast_slice(u64_slice);
    result
}
