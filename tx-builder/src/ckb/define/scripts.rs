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
        code_hash: h256!("0x08b290d4fe2d208e290cd094bdc6dacb52bff41b6dc342722f71a0183cbfe9b4"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0xb236e0350ecd161c99661330be6c8502bdb3ca293f08148c87061e45f969f006"),
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
        tx_hash: h256!("0x80fd02144cdfc5e70e97642653768e3df5fe0ef22d70802b770081e161adad0b"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref CHECKPOINT_TYPE_DEVNET: Script = Script {
        code_hash: h256!("0x1a201ec517550216c71c0e5b587828911f00d4bbd68ce07d19122314f62d37e4"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x34b31387e7b87905c336b477314711ca63f10d3ff53a9ed080010c5488f3ffec"),
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
        tx_hash: h256!("0x456f990bd454340337c751501ae54857e9cab58db0c9bb5e00b9ca57806be0a8"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref METADATA_TYPE_DEVNET: Script = Script {
        code_hash: h256!("0xfe4364f856739ba52f79bcb39dd0848267c87c46d4e82b168e21609b243a96bb"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x754ad49e8a393e0d6f65f0a4fbd2a1f623a52fb8363dd0fcc921e5511ff1719f"),
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
        tx_hash: h256!("0x9acf25c1ee3bed7fab56cc3189412b3f50375a0cefae3ca66dd1052e191f27ce"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref STAKE_LOCK_DEVNET: Script = Script {
        code_hash: h256!("0x42cea6c9708371ec9b91d19dacab6d4ac71029d36affac025f21cd8ddb1237bb"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0xd457bca47654c28b2b1417b87e875643857f4e27e55379f470e6182df59aecc9"),
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
        tx_hash: h256!("0xb96279b9ca842a995aa822e20416b0c6116042e438597e7abd1e39d769247600"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref STAKE_SMT_TYPE_DEVNET: Script = Script {
        code_hash: h256!("0x5844f7c21dae12d096deaf07a346bd611211658b17ebd63230c44830d7348e45"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x88d258b0791b54c7484a20ac57c28552fb94e2e64aa91d2b828933420ab0d454"),
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
        tx_hash: h256!("0xd93ddbb975c2fb5c8a12196c981ed76a0194b03915c9dd6a09adc83a8b09af6c"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref DELEGATE_LOCK_DEVNET: Script = Script {
        code_hash: h256!("0xe464566d6d99e58d4a1c0074a00bf7648218c81b55f3efca288c2c22eef0e6f5"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0xe5cd04a1f39bade919a976bacc1f4615bc06bd6e3a56a8f078c7620e4a93bde1"),
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
        tx_hash: h256!("0xa2785d526e20fdcd9205b4eeabee21fc1e1bbbd82a9a660b426f8e3c578795e3"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref DELEGATE_SMT_TYPE_DEVNET: Script = Script {
        code_hash: h256!("0xc3be385ece4c7dfc742087cd79547a988d4f4bfa9ceabaf1276a65d1d4446d3a"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x99d61942a42ac4a63d0e399085e9519b9ff04014a069c53142453471d705b8fb"),
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
        tx_hash: h256!("0xd85cc1a8e76476c606f8afa17a7e54d11aa4e89e9b391defd8755e50ebc5a0f5"),
        index: 0,
        dep_type: DepType::Code,
    };
    pub static ref WITHDRAW_LOCK_DEVNET: Script = Script {
        code_hash: h256!("0xd2ce87ebc56f9229574fe0ec618afd168232480d93db2814c3bfcf36df163884"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0x3b0e35d2155cb0a2f05a9db06abc1dba1ba0d0c2f33a42e5aed1f4195034bc42"),
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
        code_hash: h256!("0x30153c953e7a6e2f3394926b42e68dbdb7616eb4ea88f154e8986878ed0d0e0e"),
        hash_type: ScriptHashType::Type,
        tx_hash: h256!("0xd9f0fd4682ce3e4d9ab20e5629d7edd49b08a9e0da2b37ba40c2db0a1f280dcc"),
        index: 0,
        dep_type: DepType::Code,
    };
}
