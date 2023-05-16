use ckb_sdk::unlock::OmniLockConfig;

use ckb_types::core::ScriptHashType;
use ckb_types::packed::Script;
use ckb_types::prelude::{Builder, Entity, Pack};
use ckb_types::{h256, H160, H256};

// reference : https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0042-omnilock/0042-omnilock.md#notes
static OMNILOCK_CODE_HASH_MIRANA: H256 =
    h256!("0x9b819793a64463aed77c615d6cb226eea5487ccfc0783043a587254cda2b6f26");
static OMNILOCK_CODE_HASH_PUDGE: H256 =
    h256!("0xf329effd1c475a2978453c8600e1eaf0bc2087ee093c3ee64cc96ec6847752cb");

#[allow(unused)]
pub enum Chain {
    Mirana,
    Pudge,
}

#[allow(unused)]
pub fn build_omnilock_script(addr: &H160, chain: Chain) -> Script {
    let cfg = OmniLockConfig::new_ethereum(addr.clone());
    let omnilock_code_hash = match chain {
        Chain::Mirana => OMNILOCK_CODE_HASH_MIRANA.clone(),
        Chain::Pudge => OMNILOCK_CODE_HASH_PUDGE.clone(),
    };

    Script::new_builder()
        .code_hash(omnilock_code_hash.pack())
        .hash_type(ScriptHashType::Type.into())
        .args(cfg.build_args().pack())
        .build()
}
