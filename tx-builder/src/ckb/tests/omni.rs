#[cfg(test)]
mod tests {
    use ckb_types::h256;

    use common::types::tx_builder::NetworkType;

    use crate::ckb::utils::omni::*;
    use crate::ckb::utils::script::omni_eth_lock;

    #[test]
    fn omni() {
        // faucet: https://faucet.nervos.org/

        let test_key = h256!("0x13b08bb054d5dd04013156dced8ba2ce4d8cc5973e10d905a228ea1abc267e60");
        println!(
            "ckb address: {}",
            omni_eth_ckb_address(&NetworkType::Testnet, test_key.clone()).unwrap()
        );

        let addr = omni_eth_address(test_key).unwrap();
        println!("eth address: {}", addr);

        let lock = omni_eth_lock(&NetworkType::Testnet, &addr);
        println!("lock: {}", lock);

        let test_key = h256!("0x13b08bb054d5dd04013156dced8ba2ce4d8cc5973e10d905a228ea1abc267e63");
        println!(
            "ckb address: {}",
            omni_eth_ckb_address(&NetworkType::Testnet, test_key).unwrap()
        );
    }
}
