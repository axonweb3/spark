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
    // https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0042-omnilock/0042-omnilock.md#notes
    pub static ref OMNI_LOCK_MAINNET: Script = Script {
        code_hash: h256!("0x9b819793a64463aed77c615d6cb226eea5487ccfc0783043a587254cda2b6f26"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0xdfdb40f5d229536915f2d5403c66047e162e25dedd70a79ef5164356e1facdc8"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref OMNI_LOCK_TESTNET: Script = Script {
        code_hash: h256!("0xf329effd1c475a2978453c8600e1eaf0bc2087ee093c3ee64cc96ec6847752cb"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x27b62d8be8ed80b9f56ee0fe41355becdb6f6a40aeba82d3900434f43b1c8b60"),
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

    // https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0025-simple-udt/0025-simple-udt.md
    pub static ref SUDT_MAINNET: Script = Script {
        code_hash: h256!("0x5e7a36a77e68eecc013dfa2fe6a23f3b6c344b04005808694ae6dd45eea4cfd5"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0xc7813f6a415144643970c2e88e0bb6ca6a8edc5dd7c1022746f628284a9936d5"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref SUDT_TESTNET: Script = Script {
        code_hash: h256!("0xc5e5dcf215925f7ef4dfaf5f4b4f105bc321c02776d6e7d52a1db3fcd9d011a4"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0xe12877ebd2c3c364dc46c5c992bcfaf4fee33fa13eebdf82c591fc9825aab769"),
        index: 0,
        dep_type: DepType::Code,
    };

    // todo: mainnet
    pub static ref XUDT_MAINNET: Script = Script {
        code_hash: h256!("0x25c29dc317811a6f6f3985a7a9ebc4838bd388d19d0feeecf0bcd60f6c0975bb"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0xbf6fb538763efec2a70a6a3dcb7242787087e1030c4e7d86585bc63a9d337f5f"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref XUDT_TESTNET: Script = Script {
        code_hash: h256!("0x25c29dc317811a6f6f3985a7a9ebc4838bd388d19d0feeecf0bcd60f6c0975bb"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0xbf6fb538763efec2a70a6a3dcb7242787087e1030c4e7d86585bc63a9d337f5f"),
        index: 0,
        dep_type: DepType::Code,
    };

    // todo: main net
    pub static ref ALWAYS_SUCCESS_MAINNET: Script = Script {
        code_hash: h256!("0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x71a7ba8fc96349fea0ed3a5c47992e3b4084b031a42264a018e0072e8172e46c"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref ALWAYS_SUCCESS_TESTNET: Script = Script {
        code_hash: h256!("0x00000000000000000000000000000000000000000000000000545950455f4944"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x842380984bff8b2c7bbb8fd8886bd6784795f2f8ad140e4e2b41d771fa27314d"),
        index: 0,
        dep_type: DepType::Code,
    };

    // todo: main net
    pub static ref SELECTION_LOCK_MAINNET: Script = Script {
        code_hash: h256!("0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x71a7ba8fc96349fea0ed3a5c47992e3b4084b031a42264a018e0072e8172e46c"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref SELECTION_LOCK_TESTNET: Script = Script {
        code_hash: h256!("0x11a44037fd9164a6d20a37b00e90a9ba9dc06e79dd45d243f25cd5d405f9e3e8"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0xa4ee63a2c8694b2c4ab97e0ac6dbdd8929ece7f5a59b2d194006973c4dc2bd08"),
        index: 0,
        dep_type: DepType::Code,
    };

    // todo: main net
    pub static ref CHECKPOINT_TYPE_MAINNET: Script = Script {
        code_hash: h256!("0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x71a7ba8fc96349fea0ed3a5c47992e3b4084b031a42264a018e0072e8172e46c"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref CHECKPOINT_TYPE_TESTNET: Script = Script {
        code_hash: h256!("0x7c80a24fa3540cd4bc10a905580fb1907c87c5b62aaeb375e05577b4f8232a72"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x6bb9f0a101a24c1298aafb7ae1b4afa978631a07fa9dc15cd5dd9a5e10a400ed"),
        index: 0,
        dep_type: DepType::Code,
    };

    // todo: main net
    pub static ref METADATA_TYPE_MAINNET: Script = Script {
        code_hash: h256!("0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x71a7ba8fc96349fea0ed3a5c47992e3b4084b031a42264a018e0072e8172e46c"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref METADATA_TYPE_TESTNET: Script = Script {
        code_hash: h256!("0x30bdedc605cdb0b80f7f328c803d6059f0ad7bdeb0ccb8f44019502ac03b68a2"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x880c537b0be8b497f2cc01bb6d906da8d722857595f3ee3ada565c911ad11256"),
        index: 0,
        dep_type: DepType::Code,
    };

    // todo: main net
    pub static ref STAKE_LOCK_MAINNET: Script = Script {
        code_hash: h256!("0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x71a7ba8fc96349fea0ed3a5c47992e3b4084b031a42264a018e0072e8172e46c"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref STAKE_LOCK_TESTNET: Script = Script {
        code_hash: h256!("0x30bdedc605cdb0b80f7f328c803d6059f0ad7bdeb0ccb8f44019502ac03b68a2"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x880c537b0be8b497f2cc01bb6d906da8d722857595f3ee3ada565c911ad11256"),
        index: 0,
        dep_type: DepType::Code,
    };
}
