use axon_tools::types::{AxonBlock, Metadata, Proof, Vote, H256};
use bit_vec::BitVec;
use blst::min_pk::{AggregatePublicKey, AggregateSignature, SecretKey};
use ckb_types::H256 as CH256;
use serde::de::DeserializeOwned;
use tiny_keccak::{Hasher, Keccak};

use common::types::tx_builder::{Proof as TProof, Proposal as TProposal, Validator as TValidator};

const DST: &str = "BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RONUL";

fn read_json<T: DeserializeOwned>(path: &str) -> T {
    let json = std::fs::read_to_string(path).unwrap();
    serde_json::from_str(&json).unwrap()
}

#[allow(dead_code)]
pub fn mock_axon_proposal() -> TProposal {
    let block: AxonBlock = read_json("./src/mock/data/block.json");

    let previous_state_root =
        hex::decode("3ae76798c8eaaf3005455c254b7ca499b0de32cf5fdf0d42e967059806d93a37").unwrap();

    TProposal {
        prev_hash:                block.header.prev_hash,
        proposer:                 block.header.proposer,
        prev_state_root:          H256::from_slice(&previous_state_root),
        transactions_root:        block.header.transactions_root,
        signed_txs_hash:          block.header.signed_txs_hash,
        timestamp:                block.header.timestamp,
        number:                   block.header.number,
        proof:                    convert_proof(block.header.proof),
        call_system_script_count: block.header.call_system_script_count,
        tx_hashes:                block.tx_hashes,
    }
}

#[allow(dead_code)]
pub fn mock_axon_proof() -> TProof {
    let proof: Proof = read_json("./src/mock/data/proof.json");
    convert_proof(proof)
}

#[allow(dead_code)]
fn convert_proof(proof: Proof) -> TProof {
    TProof {
        number:     proof.number,
        round:      proof.round,
        block_hash: proof.block_hash,
        signature:  proof.signature,
        bitmap:     proof.bitmap,
    }
}

#[allow(dead_code)]
pub fn mock_axon_validators() -> Vec<TValidator> {
    let metadata: Metadata = read_json("./src/mock/data/metadata.json");

    let mut validators = metadata
        .verifier_list
        .into_iter()
        .map(|v| TValidator {
            bls_pub_key: v.bls_pub_key,
            address: ckb_types::H160::from_slice(v.address.as_bytes()).unwrap(),
            propose_weight: v.propose_weight,
            vote_weight: v.vote_weight,
            ..Default::default()
        })
        .collect::<Vec<_>>();

    validators.sort();
    validators
}

#[allow(dead_code)]
pub fn mock_axon_proposal_v2() -> TProposal {
    TProposal::default()
}

#[allow(dead_code)]
pub fn mock_axon_proof_v2(priv_keys: &Vec<CH256>) -> TProof {
    let block_hash = keccak_256(&TProposal::default().bytes());

    let vote = Vote {
        height:     200,
        round:      100,
        vote_type:  2,
        block_hash: bytes::Bytes::from(block_hash.as_slice().to_owned()),
    };
    let message = keccak_256(rlp::encode(&vote).as_ref());

    let bls_keypairs = priv_keys
        .iter()
        .map(|k| gen_bls_keypair(k.as_bytes()))
        .collect::<Vec<_>>();
    let signature = gen_bls_signature(message.as_ref(), &bls_keypairs);

    let bitmap = BitVec::from_elem(priv_keys.len(), true);

    TProof {
        number:     200,
        round:      100,
        block_hash: block_hash.into(),
        signature:  bytes::Bytes::from(signature.as_slice().to_owned()),
        bitmap:     bitmap.to_bytes().into(),
    }
}

#[allow(dead_code)]
pub fn mock_axon_validators_v2(priv_keys: &[CH256]) -> Vec<TValidator> {
    priv_keys
        .iter()
        .map(|k| TValidator {
            bls_pub_key: gen_bls_keypair(k.as_bytes()).1.into(),
            ..Default::default()
        })
        .collect::<Vec<_>>()
}

#[allow(dead_code)]
fn gen_bls_keypair(privkey: &[u8]) -> (SecretKey, Vec<u8>) {
    let privkey = SecretKey::key_gen(privkey, &[]).unwrap();
    let pubkey = privkey.sk_to_pk();
    (privkey, pubkey.compress().to_vec())
}

#[allow(dead_code)]
fn gen_bls_signature(message: &[u8], bls_keypairs: &[(SecretKey, Vec<u8>)]) -> [u8; 96] {
    let mut signatures = vec![];
    let mut pubkeys = vec![];

    for (privkey, _) in bls_keypairs {
        let signature = privkey.sign(message, DST.as_bytes(), &[]);
        let pubkey = privkey.sk_to_pk();
        signatures.push(signature);
        pubkeys.push(pubkey);
    }

    let signatures = signatures.iter().collect::<Vec<_>>();
    let signature = AggregateSignature::aggregate(signatures.as_slice(), true)
        .unwrap()
        .to_signature();

    let pubkeys = pubkeys.iter().collect::<Vec<_>>();
    let pubkey = AggregatePublicKey::aggregate(&pubkeys, false)
        .unwrap()
        .to_public_key();

    let result = signature.verify(true, message, DST.as_bytes(), &[], &pubkey, false);

    assert!(
        result == blst::BLST_ERROR::BLST_SUCCESS,
        "pubkeys not match signatures"
    );

    signature.compress()
}

#[allow(dead_code)]
fn keccak_256(data: &[u8]) -> [u8; 32] {
    let mut ret = [0u8; 32];
    let mut hasher = Keccak::v256();
    hasher.update(data);
    hasher.finalize(&mut ret);
    ret
}
