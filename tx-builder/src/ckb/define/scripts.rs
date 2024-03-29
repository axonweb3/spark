use ckb_types::core::{DepType, ScriptHashType};
use ckb_types::{h256, H256};

pub struct Script {
    pub code_hash: H256,
    pub hash_type: ScriptHashType,
    pub tx_hash:   H256,
    pub index:     u32,
    pub dep_type:  DepType,
}

// The following stores the informations (tx, cell, type id) that contains
// actual code of CKB contracts needed by Axon-Based chain
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
        code_hash: h256!("0x08b290d4fe2d208e290cd094bdc6dacb52bff41b6dc342722f71a0183cbfe9b4"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0xe9b99b660f7860190f526721ee861ffe74431ce619de71439ca56309d438ed20"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref SELECTION_LOCK_DEVNET: Script = Script {
        code_hash: h256!("0xf818dddc491e865686e95b8c979f61aa4a2a67a11c19e6544f03f043887e81c3"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0xfd8e0bb72aa8513fdc2952662f4f959ca0f115dcddc25a89a5f0ee403c20a94c"),
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
        tx_hash: h256!("0xdaaecd3476a2d5ef105c23b12330e97cc9c4f319d16e73b8bad35b84752c3905"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref CHECKPOINT_TYPE_DEVNET: Script = Script {
        code_hash: h256!("0x1a201ec517550216c71c0e5b587828911f00d4bbd68ce07d19122314f62d37e4"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0xc47897111863ac449db6e56983849810aac9fb6c14d3a0436a385840154107c2"),
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
        code_hash: h256!("0x2c8f63ac17c1e5e660dddbf49e88994cd1c49d4d6e99e7a7fd3f8879700d3cd1"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0xeb1e9be549b9e7fd2acb0c78abacc29b321f40c2d964c854ab0fd48d8111a3fc"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref METADATA_TYPE_DEVNET: Script = Script {
        code_hash: h256!("0xfe4364f856739ba52f79bcb39dd0848267c87c46d4e82b168e21609b243a96bb"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x2c4386343a7cb3527499e2c81744e6b77942c8481820613c840c0f1313981aef"),
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
        tx_hash: h256!("0x698078712359628eb7c33458aef917ab3a879f0a9a15eb8f08fef31eb07b98a9"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref STAKE_LOCK_DEVNET: Script = Script {
        code_hash: h256!("0x42cea6c9708371ec9b91d19dacab6d4ac71029d36affac025f21cd8ddb1237bb"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0xf251cc51f9da046805a1ac494a0a1aa980677ba9e76c0bb8019714a1ec4b22b6"),
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
        tx_hash: h256!("0x884fdb2409323d8c660b1c7f85089a0ccd86ab18e11012fbee6582b49243d7b3"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref STAKE_SMT_TYPE_DEVNET: Script = Script {
        code_hash: h256!("0x5844f7c21dae12d096deaf07a346bd611211658b17ebd63230c44830d7348e45"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0xd10a08401dd9c479f9c6fd6c7bbfa1c195393051a55ea72db9f7ea8f1c97eb80"),
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
        tx_hash: h256!("0xe13178be70ff5980ef98c4706b22b684e58f7e57d61bf6d8a98b08e7b328021c"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref DELEGATE_REQUIREMENT_TYPE_DEVNET: Script = Script {
        code_hash: h256!("0x3bdba036868c95873025a221c46802847a7d79a796632a58cf34dd81f15bf490"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x95afa4733a341988028088049f790d2f56c3814f6cbd813ce5257d721d6d249f"),
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
        tx_hash: h256!("0xdc505da954b67d6c55fb57e11612d6152aec0cd85cd274512014a89d10c228c7"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref DELEGATE_LOCK_DEVNET: Script = Script {
        code_hash: h256!("0xe464566d6d99e58d4a1c0074a00bf7648218c81b55f3efca288c2c22eef0e6f5"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0xb63bd1f8f347a602113f0e730c86ba1c1206a275b1b3523323f8376fe1fcc4c5"),
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
        tx_hash: h256!("0x96a04667611ef188e7d3b6d2cd24892c3c36f0066bf6c8d514f8e4c7b6d1d071"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref DELEGATE_SMT_TYPE_DEVNET: Script = Script {
        code_hash: h256!("0xc3be385ece4c7dfc742087cd79547a988d4f4bfa9ceabaf1276a65d1d4446d3a"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0xee786d36ae1682549954c16383ecc60b15eaffd851feb307f12282c2019a9040"),
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
        tx_hash: h256!("0x93569ccf3a2ebcebedd515b82500ba077fc4f072183e54bb54c9696473b9bf6f"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref WITHDRAW_LOCK_DEVNET: Script = Script {
        code_hash: h256!("0xd2ce87ebc56f9229574fe0ec618afd168232480d93db2814c3bfcf36df163884"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x9d78d411b2e5e8cd09c9d7b93fa7a92c3c4732cb742d8551c5ae439ec25ed2e6"),
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
        tx_hash: h256!("0xc18e516d69e28842e350113726edf878184d20e74873129894435bd01f578101"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref REWARD_SMT_TYPE_DEVNET: Script = Script {
        code_hash: h256!("0x30153c953e7a6e2f3394926b42e68dbdb7616eb4ea88f154e8986878ed0d0e0e"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x525b57e643e1da37157f8e41638ddc6b05149491b1c67f02980005585f72a553"),
        index: 0,
        dep_type: DepType::Code,
    };
}
