use axon_tools::types::{AxonBlock, Metadata, Proof, H256};
use serde::de::DeserializeOwned;

use common::types::tx_builder::{Proof as TProof, Proposal as TProposal, Validator as TValidator};

fn read_json<T: DeserializeOwned>(path: &str) -> T {
    let json = std::fs::read_to_string(path).unwrap();
    serde_json::from_str(&json).unwrap()
}

pub fn mock_axon_proof() -> TProof {
    let proof: Proof = read_json("./src/mock/data/proof.json");
    convert_proof(proof)
}

fn convert_proof(proof: Proof) -> TProof {
    TProof {
        number:     proof.number,
        round:      proof.round,
        block_hash: proof.block_hash,
        signature:  proof.signature,
        bitmap:     proof.bitmap,
    }
}

pub fn mock_axon_validators() -> Vec<TValidator> {
    let metadata: Metadata = read_json("./src/mock/data/metadata.json");

    metadata
        .verifier_list
        .iter()
        .map(|v| TValidator {
            bls_pub_key: v.bls_pub_key.clone(),
            address: ckb_types::H160::from_slice(v.address.as_bytes()).unwrap(),
            propose_weight: v.propose_weight,
            vote_weight: v.vote_weight,
            ..Default::default()
        })
        .collect::<Vec<_>>()
}

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
        gas_limit:                block.header.gas_limit,
        extra_data:               block.header.extra_data,
        mixed_hash:               block.header.mixed_hash,
        base_fee_per_gas:         block.header.base_fee_per_gas,
        proof:                    convert_proof(block.header.proof),
        chain_id:                 block.header.chain_id,
        call_system_script_count: block.header.call_system_script_count,
        tx_hashes:                block.tx_hashes,
    }
}
