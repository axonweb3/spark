use ckb_types::packed::{Uint128Reader, Uint64Reader};
use ckb_types::prelude::{Entity, Pack, Reader, Unpack};
use ckb_types::{H160, H256};
use molecule::prelude::Byte;

use axon_types::basic::{Byte32, Identity, Uint128, Uint16, Uint32, Uint64};

pub fn new_u128(v: &[u8]) -> u128 {
    Uint128Reader::new_unchecked(v).unpack()
}

pub fn to_u128(v: Uint128) -> u128 {
    Uint128Reader::new_unchecked(v.as_slice()).unpack()
}

pub fn to_u64(v: Uint64) -> u64 {
    Uint64Reader::new_unchecked(v.as_slice()).unpack()
}

pub fn to_bool(v: Byte) -> bool {
    v.as_slice()[0].eq(&1)
}

pub fn to_uint128(v: u128) -> Uint128 {
    Uint128::new_unchecked(v.pack().as_bytes())
}

pub fn to_uint64(v: u64) -> Uint64 {
    Uint64::new_unchecked(bytes::Bytes::from(v.to_le_bytes().to_vec()))
}

pub fn to_uint32(v: u32) -> Uint32 {
    Uint32::new_unchecked(bytes::Bytes::from(v.to_le_bytes().to_vec()))
}

pub fn to_uint16(v: u16) -> Uint16 {
    Uint16::new_unchecked(bytes::Bytes::from(v.to_le_bytes().to_vec()))
}

pub fn to_h160(v: Identity) -> H160 {
    H160::from_slice(v.as_slice()).expect("imposible")
}

pub fn to_byte32(v: H256) -> Byte32 {
    Byte32::from_slice(v.as_bytes()).expect("imposible")
}
