use anyhow::anyhow;
use anyhow::Result;
use bytes::Bytes;
use ckb_types::H256;
use common::types::tx_builder::TypeIds as CTypeIds;
use serde::{Deserialize, Serialize};

const HEX_PREFIX: &str = "0x";
const HEX_PREFIX_UPPER: &str = "0X";

pub type TypeId = Privkey;

#[derive(Clone, Debug, Deserialize)]
pub struct PrivKeys {
    pub seeder_privkey:  Privkey,
    pub staker_privkeys: Vec<Privkey>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TypeIds {
    pub selection_type_id:    TypeId,
    pub issue_type_id:        TypeId,
    pub checkpoint_type_id:   TypeId,
    pub metadata_type_id:     TypeId,
    pub stake_smt_type_id:    TypeId,
    pub delegate_smt_type_id: TypeId,
    pub reward_smt_type_id:   TypeId,
    pub xudt_owner:           TypeId,
}

impl From<CTypeIds> for TypeIds {
    fn from(v: CTypeIds) -> Self {
        Self {
            selection_type_id:    TypeId::new(&v.selection_type_id.to_string()),
            issue_type_id:        TypeId::new(&v.issue_type_id.to_string()),
            checkpoint_type_id:   TypeId::new(&v.checkpoint_type_id.to_string()),
            metadata_type_id:     TypeId::new(&v.metadata_type_id.to_string()),
            stake_smt_type_id:    TypeId::new(&v.stake_smt_type_id.to_string()),
            delegate_smt_type_id: TypeId::new(&v.delegate_smt_type_id.to_string()),
            reward_smt_type_id:   TypeId::new(&v.reward_smt_type_id.to_string()),
            xudt_owner:           TypeId::new(&v.xudt_owner.to_string()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Privkey(Hex);

impl Privkey {
    pub fn new(s: &str) -> Self {
        Self(Hex::from_string(s.to_owned()).unwrap())
    }

    pub fn inner(self) -> Hex {
        self.0
    }

    pub fn into_h256(self) -> Result<H256> {
        let key = self.inner().as_bytes();
        H256::from_slice(&key).map_err(|e| anyhow!(e.to_string()))
    }
}

impl serde::Serialize for Privkey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Privkey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct PVisitor;

        impl<'de> serde::de::Visitor<'de> for PVisitor {
            type Value = Privkey;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("Expect a hex string")
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match Hex::from_string(v.clone()) {
                    Ok(v) => Ok(Privkey(v)),
                    Err(_) => {
                        let key = std::env::var(v)
                            .map_err(|e| serde::de::Error::custom(e.to_string()))?;
                        Hex::from_string(key)
                            .map(Privkey)
                            .map_err(|e| serde::de::Error::custom(e.to_string()))
                    }
                }
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match Hex::from_string(v.to_string()) {
                    Ok(v) => Ok(Privkey(v)),
                    Err(_) => {
                        let key = std::env::var(v)
                            .map_err(|e| serde::de::Error::custom(e.to_string()))?;
                        Hex::from_string(key)
                            .map(Privkey)
                            .map_err(|e| serde::de::Error::custom(e.to_string()))
                    }
                }
            }
        }

        deserializer.deserialize_string(PVisitor)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hex(String);

impl Hex {
    pub fn from_string(s: String) -> Result<Self> {
        let s = if Self::is_prefixed(s.as_str()) {
            s
        } else {
            HEX_PREFIX.to_string() + &s
        };

        let _ = hex_decode(&s[2..])?;
        Ok(Hex(s))
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

pub fn hex_decode(src: &str) -> Result<Vec<u8>> {
    if src.is_empty() {
        return Ok(Vec::new());
    }

    let src = if src.starts_with("0x") {
        src.split_at(2).1
    } else {
        src
    };

    let src = src.as_bytes();
    let mut ret = vec![0u8; src.len() / 2];
    faster_hex::hex_decode(src, &mut ret)?;

    Ok(ret)
}
