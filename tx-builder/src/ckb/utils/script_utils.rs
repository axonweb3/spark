use ckb_sdk::unlock::OmniLockConfig;
use ckb_sdk::util::blake160;

use ckb_types::core::ScriptHashType;
use ckb_types::packed::Script;
use ckb_types::prelude::{Builder, Entity, Pack};
use ckb_types::{H160, H256};

use hex_literal::hex;

#[allow(unused)]
pub enum Chain {
    Mirana,
    Pudge,
}

#[allow(unused)]
pub fn build_omnilock_script(addr: &H160, chain: Chain) -> Script {
    let cfg = OmniLockConfig::new_pubkey_hash(blake160(addr.as_bytes()));
    // reference : https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0042-omnilock/0042-omnilock.md#notes
    let hash = match chain {
        Chain::Mirana => hex!("9b819793a64463aed77c615d6cb226eea5487ccfc0783043a587254cda2b6f26"),
        Chain::Pudge => hex!("f329effd1c475a2978453c8600e1eaf0bc2087ee093c3ee64cc96ec6847752cb"),
    };
    let omnilock_code_hash = H256::from(hash);

    Script::new_builder()
        .code_hash(omnilock_code_hash.pack())
        .hash_type(ScriptHashType::Data1.into())
        .args(cfg.build_args().pack())
        .build()
}
