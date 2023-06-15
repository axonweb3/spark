use std::fmt;

use crate::utils::codec::{deserialize_uint, hex_decode, hex_encode};
use bytes::Bytes;

use ethereum_types::{Bloom, H160, H256, H64, U256};
use serde::{de, Deserialize, Serialize};

const HEX_PREFIX: &str = "0x";
const HEX_PREFIX_UPPER: &str = "0X";

pub type Hash = H256;
pub type MerkleRoot = Hash;
pub type BlockNumber = u64;

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub header:    Header,
    pub tx_hashes: Vec<Hash>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq, Eq)]
pub struct Header {
    pub prev_hash:                Hash,
    pub proposer:                 H160,
    pub state_root:               MerkleRoot,
    pub transactions_root:        MerkleRoot,
    pub signed_txs_hash:          Hash,
    pub receipts_root:            MerkleRoot,
    pub log_bloom:                Bloom,
    pub difficulty:               U256,
    #[serde(deserialize_with = "deserialize_uint")]
    pub timestamp:                u64,
    #[serde(deserialize_with = "deserialize_uint")]
    pub number:                   BlockNumber,
    pub gas_used:                 U256,
    pub gas_limit:                U256,
    pub extra_data:               Bytes,
    pub mixed_hash:               Option<Hash>,
    pub nonce:                    H64,
    pub base_fee_per_gas:         U256,
    pub proof:                    Proof,
    #[serde(deserialize_with = "deserialize_uint")]
    pub call_system_script_count: u64,
    #[serde(deserialize_with = "deserialize_uint")]
    pub chain_id:                 u64,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq, Eq)]
pub struct Proof {
    #[serde(deserialize_with = "deserialize_uint")]
    pub number:     u64,
    #[serde(deserialize_with = "deserialize_uint")]
    pub round:      u64,
    pub block_hash: Hash,
    pub signature:  Bytes,
    pub bitmap:     Bytes,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, Copy, PartialEq, Eq)]
pub struct MetadataVersion {
    #[serde(deserialize_with = "deserialize_uint")]
    pub start: BlockNumber,
    #[serde(deserialize_with = "deserialize_uint")]
    pub end:   BlockNumber,
}

impl MetadataVersion {
    pub fn new(start: BlockNumber, end: BlockNumber) -> Self {
        MetadataVersion { start, end }
    }

    pub fn contains(&self, number: BlockNumber) -> bool {
        self.start <= number && number <= self.end
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ProposeCount {
    pub address: H160,
    #[serde(deserialize_with = "deserialize_uint")]
    pub count:   u64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Default)]
pub struct ValidatorExtend {
    pub bls_pub_key:    Hex,
    pub pub_key:        Hex,
    pub address:        H160,
    #[serde(deserialize_with = "deserialize_uint")]
    pub propose_weight: u64,
    #[serde(deserialize_with = "deserialize_uint")]
    pub vote_weight:    u64,
}

impl std::fmt::Debug for ValidatorExtend {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let bls_pub_key = self.bls_pub_key.as_string_trim0x();
        let pk = if bls_pub_key.len() > 8 {
            unsafe { bls_pub_key.get_unchecked(0..8) }
        } else {
            bls_pub_key.as_str()
        };

        write!(
            f,
            "bls public key {:?}, public key {:?}, address {:?} propose weight {}, vote weight {}",
            pk, self.pub_key, self.address, self.propose_weight, self.vote_weight
        )
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq, Eq)]
pub struct Metadata {
    pub version:         MetadataVersion,
    #[serde(deserialize_with = "deserialize_uint")]
    pub epoch:           u64,
    #[serde(deserialize_with = "deserialize_uint")]
    pub gas_limit:       u64,
    #[serde(deserialize_with = "deserialize_uint")]
    pub gas_price:       u64,
    #[serde(deserialize_with = "deserialize_uint")]
    pub interval:        u64,
    pub verifier_list:   Vec<ValidatorExtend>,
    #[serde(deserialize_with = "deserialize_uint")]
    pub propose_ratio:   u64,
    #[serde(deserialize_with = "deserialize_uint")]
    pub prevote_ratio:   u64,
    #[serde(deserialize_with = "deserialize_uint")]
    pub precommit_ratio: u64,
    #[serde(deserialize_with = "deserialize_uint")]
    pub brake_ratio:     u64,
    #[serde(deserialize_with = "deserialize_uint")]
    pub tx_num_limit:    u64,
    #[serde(deserialize_with = "deserialize_uint")]
    pub max_tx_size:     u64,
    pub propose_counter: Vec<ProposeCount>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LatestCheckPointInfo {
    pub state_root:          MerkleRoot,
    pub latest_block_height: BlockNumber,
    pub latest_block_hash:   Hash,
    pub timestamp:           u64,
    pub proof:               Proof,
    pub propose_count:       Vec<ProposeCount>,
}

impl LatestCheckPointInfo {
    #[allow(unused)]
    pub fn new(last_header: &Header, latest_header: &Header, last_metadata: &Metadata) -> Self {
        Self {
            state_root:          last_header.state_root,
            latest_block_height: last_header.number,
            latest_block_hash:   latest_header.prev_hash,
            timestamp:           last_header.timestamp,
            proof:               latest_header.proof.clone(),
            propose_count:       last_metadata.propose_counter.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hex(String);

impl Hex {
    pub fn empty() -> Self {
        Hex(String::from(HEX_PREFIX))
    }

    pub fn is_empty(&self) -> bool {
        self.0.len() == 2
    }

    pub fn encode<T: AsRef<[u8]>>(src: T) -> Self {
        let mut s = HEX_PREFIX.to_string();
        s.push_str(&hex_encode(src));
        Hex(s)
    }

    pub fn decode(s: String) -> Result<Bytes, String> {
        let s = if Self::is_prefixed(s.as_str()) {
            &s[2..]
        } else {
            s.as_str()
        };

        Ok(Bytes::from(hex_decode(s)?))
    }

    pub fn from_string(s: String) -> Result<Self, String> {
        let s = if Self::is_prefixed(s.as_str()) {
            s
        } else {
            HEX_PREFIX.to_string() + &s
        };

        let _ = hex_decode(&s[2..])?;
        Ok(Hex(s))
    }

    pub fn as_string(&self) -> String {
        self.0.to_owned()
    }

    pub fn as_string_trim0x(&self) -> String {
        (self.0[2..]).to_owned()
    }

    pub fn as_bytes(&self) -> Bytes {
        Bytes::from(hex_decode(&self.0[2..]).expect("impossible, already checked in from_string"))
    }

    fn is_prefixed(s: &str) -> bool {
        s.starts_with(HEX_PREFIX) || s.starts_with(HEX_PREFIX_UPPER)
    }
}

impl Default for Hex {
    fn default() -> Self {
        Hex(String::from("0x0000000000000000"))
    }
}

impl Serialize for Hex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Hex {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_string(HexVisitor)
    }
}

struct HexVisitor;

impl<'de> de::Visitor<'de> for HexVisitor {
    type Value = Hex;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Expect a hex string")
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Hex::from_string(v).map_err(|e| de::Error::custom(e))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Hex::from_string(v.to_owned()).map_err(|e| de::Error::custom(e))
    }
}

pub fn mock_header() -> Header {
    let header_raw = r#"{
        "prev_hash": "0xdb240f91d1cbf9a538cb7e2c1bbdcc6a8eb05bf29997cb7210649c8377d7cdf4",
        "proposer": "0x8ab0cf264df99d83525e9e11c7e4db01558ae1b1",
        "state_root": "0xfa1fb80b6c30d7f7d9c11666f4d48c304dbc2a143773eb057febf28f764d2a4c",
        "transactions_root": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
        "signed_txs_hash": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
        "receipts_root": "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
        "log_bloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000001000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
        "difficulty": "0x1",
        "timestamp": "0x648747cc",
        "number": "0xbc04",
        "gas_used": "0x0",
        "gas_limit": "0x1c9c380",
        "extra_data": "0x",
        "mixed_hash": null,
        "nonce": "0x0000000000000000",
        "base_fee_per_gas": "0x539",
        "proof": {
          "number": "0xbc03",
          "round": "0x0",
          "block_hash": "0x323915d8592e2ce856bd015e83d8e3e57106309d6eee329ef9f94f425b02aa66",
          "signature": "0xae22cd5f04b212ab6d881f8be2b593a0a2b8e5c004ee3c28e6ba42463cc645b7e27baed9b2a5bdb41a8af7833980e69e1288f72c5eac08261e1474044e389e985445885ae26e755796f10d2e09237905e2ee93a6a3fe42958727bbb2af417684",
          "bitmap": "0x80"
        },
        "call_system_script_count": "0x0",
        "chain_id": "0x7e6"
      }"#;

    serde_json::from_str(header_raw).unwrap()
}

pub fn mock_metadata() -> Metadata {
    serde_json::from_str(r#"{
        "version": {
          "start": "0x1",
          "end": "0x5f5e100"
        },
        "epoch": "0x0",
        "gas_limit": "0x3e7fffffc18",
        "gas_price": "0x1",
        "interval": "0xbb8",
        "verifier_list": [
          {
            "bls_pub_key": "0xac85bbb40347b6e06ac2dc2da1f75eece029cdc0ed2d456c457d27e288bfbfbcd4c5c19716e9b250134a0e76ce50fa22",
            "pub_key": "0x031ddc35212b7fc7ff6685b17d91f77c972535aee5c7ae5684d3e72b986f08834b",
            "address": "0x8ab0cf264df99d83525e9e11c7e4db01558ae1b1",
            "propose_weight": "0x1",
            "vote_weight": "0x1"
          }
        ],
        "propose_ratio": "0xf",
        "prevote_ratio": "0xa",
        "precommit_ratio": "0xa",
        "brake_ratio": "0xa",
        "tx_num_limit": "0x4e20",
        "max_tx_size": "0x186a0000",
        "propose_counter": [
          {
            "address": "0x8ab0cf264df99d83525e9e11c7e4db01558ae1b1",
            "count": "0xbc08"
          }
        ]
      }"#).unwrap()
}

pub fn mock_latest_check_point_info() -> LatestCheckPointInfo {
    let head = mock_header();
    let metadata = mock_metadata();
    LatestCheckPointInfo::new(&head, &head, &metadata)
}
