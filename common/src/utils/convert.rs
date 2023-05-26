use ckb_types::packed::{Byte32 as CByte32, Uint128Reader};
use ckb_types::prelude::{Entity, Pack, Reader, Unpack};
use ckb_types::{H160, H256};
use molecule::prelude::Byte;

use axon_types::basic::{Byte32, Identity, Uint128, Uint16, Uint32, Uint64};

pub fn new_u128(v: &[u8]) -> u128 {
    Uint128Reader::new_unchecked(v).unpack()
}

pub fn to_u128(v: &Uint128) -> u128 {
    let mut array: [u8; 16] = [0u8; 16];
    array.copy_from_slice(v.as_slice());
    u128::from_le_bytes(array)
}

pub fn to_u64(v: &Uint64) -> u64 {
    let mut array: [u8; 8] = [0u8; 8];
    array.copy_from_slice(v.as_slice());
    u64::from_le_bytes(array)
}

pub fn to_bool(v: &Byte) -> bool {
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

pub fn to_h160(v: &Identity) -> H160 {
    H160::from_slice(v.as_slice()).expect("imposible")
}

pub fn to_byte32(v: &H256) -> Byte32 {
    Byte32::from_slice(v.as_bytes()).expect("imposible")
}

pub fn to_h256(v: &CByte32) -> H256 {
    H256::from_slice(v.as_slice()).expect("imposible")
}

pub fn to_ckb_byte32(v: &Byte32) -> ckb_types::packed::Byte32 {
    ckb_types::packed::Byte32::new_unchecked(v.as_bytes())
}

pub fn to_axon_byte32(v: &ckb_types::packed::Byte32) -> Byte32 {
    Byte32::new_unchecked(v.as_bytes())
}

#[test]
fn test_u128() {
    let a = 100_u128;
    assert_eq!(a, to_u128(&to_uint128(a)));
}

#[test]
fn test_ckb_byte32() {
    let a = Byte32::default();
    assert_eq!(ckb_types::packed::Byte32::default(), to_ckb_byte32(&a));
}
