use std::collections::HashMap;
use std::fs;
use std::path::Path;

use sparse_merkle_tree::{
    blake2b::Blake2bHasher, default_store::DefaultStore, merkle_proof::CompiledMerkleProof,
    traits::Value, SparseMerkleTree, H256,
};

use common::types::smt::{Address, LeafValue, SmtKeyEncode, SmtValueEncode};

use crate::ROCKSDB_PATH;

type Smt = SparseMerkleTree<Blake2bHasher, LeafValue, DefaultStore<LeafValue>>;

pub fn remove_smt() {
    if Path::new(ROCKSDB_PATH).is_dir() {
        fs::remove_dir_all(ROCKSDB_PATH).unwrap();
    }
}

pub fn to_root(v: &bytes::Bytes) -> H256 {
    let mut buf = [0u8; 32];
    buf.copy_from_slice(v);
    H256::from(buf)
}

pub fn generate_smt_root(leaves: HashMap<Address, u128>) -> H256 {
    let kvs: Vec<(H256, LeafValue)> = leaves
        .into_iter()
        .map(|(k, v)| {
            (
                SmtKeyEncode::Address(k).to_h256(),
                SmtValueEncode::Amount(v).to_leaf_value(),
            )
        })
        .collect();

    let mut smt = Smt::default();
    smt.update_all(kvs).expect("update");
    *smt.root()
}

pub fn verify_proof(top_root: H256, top_proof: Vec<u8>, epoch: u64, bottom_root: H256) -> bool {
    let leaves = vec![(
        SmtKeyEncode::Epoch(epoch).to_h256(),
        SmtValueEncode::Root(bottom_root).to_leaf_value().to_h256(),
    )];

    let proof = CompiledMerkleProof(top_proof);
    proof.verify::<Blake2bHasher>(&top_root, leaves).unwrap()
}
