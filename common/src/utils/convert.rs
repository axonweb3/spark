use ckb_types::packed::{Uint128Reader, Uint32 as CUint32, Uint64 as CUint64, Uint64Reader};
use ckb_types::prelude::{Entity, Pack, Reader, Unpack};
use ckb_types::H160;
use molecule::prelude::Byte;

use axon_types::basic::{Identity, Uint128, Uint32, Uint64};

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
    let v: CUint64 = v.pack();
    Uint64::new_unchecked(v.as_bytes())
}

pub fn to_uint32(v: u32) -> Uint32 {
    let v: CUint32 = v.pack();
    Uint32::new_unchecked(v.as_bytes())
}

pub fn to_h160(v: Identity) -> H160 {
    H160::from_slice(v.as_slice()).expect("imposible")
}
