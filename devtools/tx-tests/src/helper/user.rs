use ckb_types::H256;
use common::types::tx_builder::EthAddress;
use tx_builder::ckb::helper::OmniEth;

use crate::config::types::Privkey;

pub fn gen_users(privkeys: Vec<Privkey>) -> (Vec<H256>, Vec<EthAddress>) {
    let priv_keys: Vec<H256> = privkeys
        .into_iter()
        .map(|key| key.into_h256().unwrap())
        .collect();

    let users: Vec<EthAddress> = priv_keys
        .clone()
        .into_iter()
        .map(|key| OmniEth::new(key).address().unwrap())
        .collect();

    (priv_keys, users)
}
