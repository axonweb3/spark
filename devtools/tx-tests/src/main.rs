mod config;
mod mock;
mod tx;

use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;

use crate::tx::*;

pub const PRIV_KEYS_PATH: &str = "./src/config/priv_keys.toml";
pub const TYPE_IDS_PATH: &str = "./src/config/type_ids.toml";

#[tokio::main]
async fn main() {
    let cmd = clap::Command::new("spark")
        .version(clap::crate_version!())
        .arg(
            clap::Arg::new("init")
                .short('i')
                .required(false)
                .num_args(0)
                .help("test init tx"),
        )
        .arg(
            clap::Arg::new("mint")
                .short('m')
                .required(false)
                .num_args(0)
                .help("test mint tx"),
        )
        .arg(
            clap::Arg::new("stake")
                .short('s')
                .required(false)
                .num_args(1)
                .default_value("")
                .help("test stake tx"),
        )
        .arg(
            clap::Arg::new("delegate")
                .short('d')
                .required(false)
                .num_args(1)
                .default_value("")
                .help("test delegate tx"),
        )
        .arg(
            clap::Arg::new("checkpoint")
                .short('c')
                .required(false)
                .num_args(0)
                .help("test checkpoint tx"),
        )
        .arg(
            clap::Arg::new("stake smt")
                .short('t')
                .required(false)
                .num_args(0)
                .help("test stake smt tx"),
        )
        .arg(
            clap::Arg::new("delegate smt")
                .short('e')
                .required(false)
                .num_args(0)
                .help("test delegate smt tx"),
        )
        .arg(
            clap::Arg::new("reward")
                .short('r')
                .required(false)
                .num_args(0)
                .help("test reward tx"),
        );

    let matches = cmd.get_matches();
    let init = matches.get_one::<bool>("init").unwrap().to_owned();
    let mint = matches.get_one::<bool>("mint").unwrap().to_owned();
    let stake = matches.get_one::<String>("stake").unwrap().as_str();
    let delegate = matches.get_one::<String>("delegate").unwrap().as_str();
    let checkpoint = matches.get_one::<bool>("checkpoint").unwrap().to_owned();
    let stake_smt = matches.get_one::<bool>("stake smt").unwrap().to_owned();
    let delegate_smt = matches.get_one::<bool>("delegate smt").unwrap().to_owned();
    let reward = matches.get_one::<bool>("reward").unwrap().to_owned();

    let ckb = CkbRpcClient::new("https://testnet.ckb.dev");

    if init {
        init_tx(&ckb).await;
    }

    if mint {
        mint_tx(&ckb).await;
    }

    if !stake.is_empty() {
        match stake {
            "first" => first_stake_tx(&ckb).await,
            "add" => add_stake_tx(&ckb).await,
            "redeem" => reedem_stake_tx(&ckb).await,
            _ => unimplemented!(),
        }
    }

    if !delegate.is_empty() {
        match delegate {
            "first" => first_delegate_tx(&ckb).await,
            "add" => add_delegate_tx(&ckb).await,
            "redeem" => reedem_delegate_tx(&ckb).await,
            _ => unimplemented!(),
        }
    }

    if checkpoint {
        checkpoint_tx(&ckb).await;
    }

    if stake_smt {
        stake_smt_tx(&ckb).await;
    }

    if delegate_smt {
        delegate_smt_tx(&ckb).await;
    }

    if reward {
        reward_tx(&ckb).await;
    }
}
