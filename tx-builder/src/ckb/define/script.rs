use ckb_types::core::{DepType, ScriptHashType};
use ckb_types::{h256, H256};

pub struct Script {
    pub code_hash: H256,
    pub hash_type: ScriptHashType,
    pub tx_hash:   H256,
    pub index:     u32,
    pub dep_type:  DepType,
}

lazy_static::lazy_static! {
    // https://github.com/nervosnetwork/light-godwoken/blob/f9153b64bdee22acb4ee20b4827dc0927c284ae6/src/light-godwoken/constants/lightGodwokenConfig.ts
    pub static ref OMNI_LOCK_MAINNET: Script = Script {
        code_hash: h256!("0x9f3aeaf2fc439549cbc870c653374943af96a0658bd6b51be8d8983183e6f52f"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0xaa8ab7e97ed6a268be5d7e26d63d115fa77230e51ae437fc532988dd0c3ce10a"),
        index: 1,
        dep_type: DepType::Code,
    };
    pub static ref OMNI_LOCK_TESTNET: Script = Script {
        code_hash: h256!("0x79f90bb5e892d80dd213439eeab551120eb417678824f282b4ffb5f21bad2e1e"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x9154df4f7336402114d04495175b37390ce86a4906d2d4001cf02c3e6d97f39c"),
        index: 0,
        dep_type: DepType::Code,
    };

    // https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0024-ckb-genesis-script-list/0024-ckb-genesis-script-list.md#secp256k1blake160
    pub static ref SECP2561_BLAKE160_MAINNET: Script = Script {
        code_hash: h256!("0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x71a7ba8fc96349fea0ed3a5c47992e3b4084b031a42264a018e0072e8172e46c"),
        index: 0,
        dep_type: DepType::DepGroup,
    };
    pub static ref SECP2561_BLAKE160_TESTNET: Script = Script {
        code_hash: h256!("0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0xf8de3bb47d055cdf460d93a2a6e1b05f7432f9777c8c474abf4eec1d4aee5d37"),
        index: 0,
        dep_type: DepType::DepGroup,
    };

    // todo
    pub static ref XUDT_MAINNET: Script = Script {
        code_hash: h256!("0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x71a7ba8fc96349fea0ed3a5c47992e3b4084b031a42264a018e0072e8172e46c"),
        index: 0,
        dep_type: DepType::DepGroup,
    };
    pub static ref XUDT_TESTNET: Script = Script {
        code_hash: h256!("0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0xf8de3bb47d055cdf460d93a2a6e1b05f7432f9777c8c474abf4eec1d4aee5d37"),
        index: 0,
        dep_type: DepType::DepGroup,
    };

    // todo
    pub static ref CANNOT_DESTROY_MAINNET: Script = Script {
        code_hash: h256!("0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x71a7ba8fc96349fea0ed3a5c47992e3b4084b031a42264a018e0072e8172e46c"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref CANNOT_DESTROY_TESTNET: Script = Script {
        code_hash: h256!("0x00000000000000000000000000000000000000000000000000545950455f4944"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x842380984bff8b2c7bbb8fd8886bd6784795f2f8ad140e4e2b41d771fa27314d"),
        index: 0,
        dep_type: DepType::Code,
    };
}
