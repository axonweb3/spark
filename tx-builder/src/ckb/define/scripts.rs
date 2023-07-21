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
    pub static ref OMNI_LOCK_DEVNET: Script = Script {
        code_hash: h256!("0x2bc282b9695f9d45511912e081aace7a21e6e3f6f5c718794e2dd0d385a3b93f"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x44f9f4463165377334443cf3aec929612b9ff4ff104a82d326fa079adae17e6b"),
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
    pub static ref SECP2561_BLAKE160_DEVNET: Script = Script {
        code_hash: h256!("0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0xe47e376dab10cc44f362b382bdcd2d80afa68ee9aa8e13992e327cf932a2e50b"),
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

    // todo: main net
    pub static ref XUDT_TYPE_MAINNET: Script = Script {
        code_hash: h256!("0x25c29dc317811a6f6f3985a7a9ebc4838bd388d19d0feeecf0bcd60f6c0975bb"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0xbf6fb538763efec2a70a6a3dcb7242787087e1030c4e7d86585bc63a9d337f5f"),
        index: 0,
        dep_type: DepType::Code,
    };
    // https://blog.cryptape.com/enhance-sudts-programmability-with-xudt
    pub static ref XUDT_TYPE_TESTNET: Script = Script {
        code_hash: h256!("0x25c29dc317811a6f6f3985a7a9ebc4838bd388d19d0feeecf0bcd60f6c0975bb"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0xbf6fb538763efec2a70a6a3dcb7242787087e1030c4e7d86585bc63a9d337f5f"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref XUDT_TYPE_DEVNET: Script = Script {
        code_hash: h256!("0x014bdba65b108c3901e479dfd301c2490f55e003d7aafb694b1bbfdc13c842b1"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0xf6a9cd762a440d2c2e31eb286ffbf8989d46e99f629f023ea4aa0accafefb3c9"),
        index: 0,
        dep_type: DepType::Code,
    };

    // todo: main net
    pub static ref ALWAYS_SUCCESS_LOCK_MAINNET: Script = Script {
        code_hash: h256!("0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x71a7ba8fc96349fea0ed3a5c47992e3b4084b031a42264a018e0072e8172e46c"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref ALWAYS_SUCCESS_LOCK_TESTNET: Script = Script {
        code_hash: h256!("0xd4edb0fa797f92bc0dcb8bcef036c55e3f591316ca4af6a5fb4cc4a5e67cb014"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x1f2d3579dcb8599e31ce71f3b471be7e1edd77c314c0942eb26d11c80d259ba9"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref ALWAYS_SUCCESS_LOCK_DEVNET: Script = Script {
        code_hash: h256!("0x2ece184496d35bc46577ec24e298f086e5e493c1263433a8baf74da1faa6721c"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0xe87862e1b0a96294a268aa9452350c68c798487ccf7a0b99af54e2a879e46dbf"),
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
    pub static ref SELECTION_LOCK_DEVNET: Script = Script {
        code_hash: h256!("0xf818dddc491e865686e95b8c979f61aa4a2a67a11c19e6544f03f043887e81c3"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x01861c5cef65389900c0f94d9f3c2e7c971ad53532327d834cb8f72dde01c072"),
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
        code_hash: h256!("0x442d5c2eb01e14db2b0acb136dd2cdda1c3515fc4085898a47dd773ca1c3d019"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0xd7d86574fcbc1bb2d0cfada3b9a39991a7399d8b33c980e5a68299f2d6360d82"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref CHECKPOINT_TYPE_DEVNET: Script = Script {
        code_hash: h256!("0x1a201ec517550216c71c0e5b587828911f00d4bbd68ce07d19122314f62d37e4"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x929cc6c65a6df652e0f1383dc40c96330037fb3491c939ac16cb9511d88d1c61"),
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
        tx_hash: h256!("0x986220a6bafe37c3d7f8e2bb4dfbb7392188d142449cf38b098ba8b1a24009b6"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref METADATA_TYPE_DEVNET: Script = Script {
        code_hash: h256!("0x35ff0cfece2bfa6352b2efb540fd2ba3e87c03f34340c6f43d4fe2c280071470"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x1954faca0be9302176a87d474c5a3488c52aff29e90bd279217ecb8e53316302"),
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
        code_hash: h256!("0x0a2adef4af62c9350eee7d31dfc2b5f340f2fa5c5d70f6834c13465cb545cde3"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x790bf6c94a9ef39ea353c98deb77a238b113580676fa2de86bc547f0d3a55a5d"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref STAKE_LOCK_DEVNET: Script = Script {
        code_hash: h256!("0x42cea6c9708371ec9b91d19dacab6d4ac71029d36affac025f21cd8ddb1237bb"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x2b3427595a18bc207139a5c1e5cb1c6f087fc432814a76f7c149cc8a135fd727"),
        index: 0,
        dep_type: DepType::Code,
    };

    // todo: main net
    pub static ref STAKE_SMT_TYPE_MAINNET: Script = Script {
        code_hash: h256!("0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x71a7ba8fc96349fea0ed3a5c47992e3b4084b031a42264a018e0072e8172e46c"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref STAKE_SMT_TYPE_TESTNET: Script = Script {
        code_hash: h256!("0x4b40fbfc384278eb1a8bcda34a08b37642d33d49a804e56185926ff6e779e01d"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x790bf6c94a9ef39ea353c98deb77a238b113580676fa2de86bc547f0d3a55a5d"),
        index: 1,
        dep_type: DepType::Code,
    };
    pub static ref STAKE_SMT_TYPE_DEVNET: Script = Script {
        code_hash: h256!("0x5844f7c21dae12d096deaf07a346bd611211658b17ebd63230c44830d7348e45"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x39635e012c080735ed92cc67b0877588f3a88244bb97364d5bd2143003d2cffe"),
        index: 0,
        dep_type: DepType::Code,
    };

    // todo: main net
    pub static ref DELEGATE_REQUIREMENT_TYPE_MAINNET: Script = Script {
        code_hash: h256!("0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x71a7ba8fc96349fea0ed3a5c47992e3b4084b031a42264a018e0072e8172e46c"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref DELEGATE_REQUIREMENT_TYPE_TESTNET: Script = Script {
        code_hash: h256!("0x1122036fd7be9796625b60a22e045fe5d03ffb2d559e86098d896645f3f356b0"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x95876df71b1c631ba6cc1abb1e2d789cebef35837f223bfd56ad7b66bc8fcb0e"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref DELEGATE_REQUIREMENT_TYPE_DEVNET: Script = Script {
        code_hash: h256!("0x3bdba036868c95873025a221c46802847a7d79a796632a58cf34dd81f15bf490"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0xdde1ed4519b068988a9006a4a9518de585cc51949a20e26d8edd6b1bd736b547"),
        index: 0,
        dep_type: DepType::Code,
    };

    // todo: main net
    pub static ref DELEGATE_LOCK_MAINNET: Script = Script {
        code_hash: h256!("0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x71a7ba8fc96349fea0ed3a5c47992e3b4084b031a42264a018e0072e8172e46c"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref DELEGATE_LOCK_TESTNET: Script = Script {
        code_hash: h256!("0x0aadf36bb5e1b60cf7e550ad9705592188b5974ed6f8eed30feb76721dc15395"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x4309cb77761457babc7d9888eda1b0c7f733700fa23639fc0686fc71ef82cde4"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref DELEGATE_LOCK_DEVNET: Script = Script {
        code_hash: h256!("0xe464566d6d99e58d4a1c0074a00bf7648218c81b55f3efca288c2c22eef0e6f5"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x473b51973dda025d5f21cf022dd64c131f48af58b4f47811be65ea5ca319227d"),
        index: 0,
        dep_type: DepType::Code,
    };

    // todo: main net
    pub static ref DELEGATE_SMT_TYPE_MAINNET: Script = Script {
        code_hash: h256!("0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x71a7ba8fc96349fea0ed3a5c47992e3b4084b031a42264a018e0072e8172e46c"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref DELEGATE_SMT_TYPE_TESTNET: Script = Script {
        code_hash: h256!("0xa04dd0442bcbecfb32451782edb53c0ac8c81927f551bb7faba98b41bdcb22b2"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x4309cb77761457babc7d9888eda1b0c7f733700fa23639fc0686fc71ef82cde4"),
        index: 1,
        dep_type: DepType::Code,
    };
    pub static ref DELEGATE_SMT_TYPE_DEVNET: Script = Script {
        code_hash: h256!("0xc3be385ece4c7dfc742087cd79547a988d4f4bfa9ceabaf1276a65d1d4446d3a"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x043960b897d4a271dca5e98228cb9227636192914bde6f206252d1beac8e9353"),
        index: 0,
        dep_type: DepType::Code,
    };

    // todo: main net
    pub static ref WITHDRAW_LOCK_MAINNET: Script = Script {
        code_hash: h256!("0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x71a7ba8fc96349fea0ed3a5c47992e3b4084b031a42264a018e0072e8172e46c"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref WITHDRAW_LOCK_TESTNET: Script = Script {
        code_hash: h256!("0xf33d2e1805347e62ff162bb8d2abf62cd386cdc9af6c455aafa4aa6ecaefbc0d"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x9221b656d96c231d619c8fe34cbc1845702fe1c1998ec8f5c395fd5abd612373"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref WITHDRAW_LOCK_DEVNET: Script = Script {
        code_hash: h256!("0xd2ce87ebc56f9229574fe0ec618afd168232480d93db2814c3bfcf36df163884"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0xcd83b7f2cc31bf831caacea4245986357220574f04b07bf7af7dc8986ea1c4dd"),
        index: 0,
        dep_type: DepType::Code,
    };

    // todo: main net
    pub static ref REWARD_SMT_TYPE_MAINNET: Script = Script {
        code_hash: h256!("0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x71a7ba8fc96349fea0ed3a5c47992e3b4084b031a42264a018e0072e8172e46c"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref REWARD_SMT_TYPE_TESTNET: Script = Script {
        code_hash: h256!("0x22b1350ab5764d255bed11c51283a8a462bcfbdf42c42eb13f4bcb8da6cbe867"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0xb937a8938d588cddf38bab8273ce474f5d29dce6d0f9ac8c2d49b9bbe25e84b7"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref REWARD_SMT_TYPE_DEVNET: Script = Script {
        code_hash: h256!("0xdae7c049e40c9988a7c39f00c1d6a72b377fe92cd8ace596001e918d7602aff5"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x51a99304d677537dcfe598f19127f88c1cd9cce1fce462772a2e904288d52bda"),
        index: 0,
        dep_type: DepType::Code,
    };
}
