#[cfg(test)]
mod tests {
    use ckb_types::h256;

    use crate::ckb::helper::ckb::omni::OmniEth;

    #[test]
    fn omni() {
        // faucet: https://faucet.nervos.org/

        let test_key = h256!("0x13b08bb054d5dd04013156dced8ba2ce4d8cc5973e10d905a228ea1abc267e60");
        let omni_eth = OmniEth::new(test_key);
        println!("ckb address: {}", omni_eth.ckb_address().unwrap());

        let addr = omni_eth.address().unwrap();
        println!("eth address: {}", addr);

        let lock = OmniEth::lock(&addr);
        println!("lock: {}", lock);

        let test_key = h256!("0x13b08bb054d5dd04013156dced8ba2ce4d8cc5973e10d905a228ea1abc267e63");
        let omni_eth = OmniEth::new(test_key);
        println!("ckb address: {}", omni_eth.ckb_address().unwrap());
    }
}
