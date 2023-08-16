use ckb_types::{h160, h256};

use crate::ckb::helper::ckb::omni::OmniEth;

#[test]
fn omni() {
    // faucet: https://faucet.nervos.org/

    let test_key = h256!("0x13b08bb054d5dd04013156dced8ba2ce4d8cc5973e10d905a228ea1abc267e60");
    let omni_eth = OmniEth::new(test_key);

    let ckb_addr = omni_eth.ckb_address().unwrap();
    assert_eq!(
        &ckb_addr,
        "ckt1qrejnmlar3r452tcg57gvq8patctcgy8acync0hxfnyka35ywafvkqgp3r7g3djanfsn5xk487e0g8juh8r5r7mdqqkzm504",
    );

    let eth_addr = omni_eth.address().unwrap();
    assert_eq!(
        eth_addr,
        h160!("0x88fc88b65d9a613a1ad53fb2f41e5cb9c741fb6d"),
    );
}
