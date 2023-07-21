mod config;
mod mock;
mod tx;

use common::types::tx_builder::NetworkType;
use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;
use tx_builder::set_network_type;

use crate::tx::*;

pub const PRIV_KEYS_PATH: &str = "./src/config/priv_keys.toml";
pub const TYPE_IDS_PATH: &str = "./src/config/type_ids.toml";

#[tokio::main]
async fn main() {
    let cmd = clap::Command::new("spark")
        .version(clap::crate_version!())
        .arg(
            clap::Arg::new("net")
                .short('n')
                .required(false)
                .num_args(1)
                .value_parser(["dev", "test", "main"])
                .default_value("test")
                .help("Switch network"),
        )
        .arg(
            clap::Arg::new("faucet")
                .short('f')
                .required(false)
                .num_args(0)
                .help("Send CKB from secp256k1 address to Omni ETH CKB address"),
        )
        .arg(
            clap::Arg::new("init")
                .short('i')
                .required(false)
                .num_args(0)
                .help("Test init tx"),
        )
        .arg(
            clap::Arg::new("mint")
                .short('m')
                .required(false)
                .num_args(0)
                .help("Test mint tx"),
        )
        .arg(
            clap::Arg::new("stake")
                .short('s')
                .required(false)
                .num_args(1)
                .value_parser(["first", "add", "redeem"])
                .help("Test stake tx"),
        )
        .arg(
            clap::Arg::new("delegate")
                .short('d')
                .required(false)
                .num_args(1)
                .value_parser(["first", "add", "redeem"])
                .help("Test delegate tx"),
        )
        .arg(
            clap::Arg::new("checkpoint")
                .short('c')
                .required(false)
                .num_args(0)
                .help("Test checkpoint tx"),
        )
        .arg(
            clap::Arg::new("stake-smt")
                .short('t')
                .required(false)
                .num_args(0)
                .help("Test stake smt tx"),
        )
        .arg(
            clap::Arg::new("delegate-smt")
                .short('e')
                .required(false)
                .num_args(0)
                .help("Test delegate smt tx"),
        )
        .arg(
            clap::Arg::new("withdraw")
                .short('w')
                .required(false)
                .num_args(0)
                .help("Test withdraw tx"),
        )
        .arg(
            clap::Arg::new("reward")
                .short('r')
                .required(false)
                .num_args(0)
                .help("Test reward tx"),
        );

    let matches = cmd.get_matches();
    let net = matches.get_one::<String>("net").unwrap().as_str();
    let faucet = *matches.get_one::<bool>("faucet").unwrap();
    let init = *matches.get_one::<bool>("init").unwrap();
    let mint = *matches.get_one::<bool>("mint").unwrap();
    let stake = matches.get_one::<String>("stake");
    let delegate = matches.get_one::<String>("delegate");
    let checkpoint = *matches.get_one::<bool>("checkpoint").unwrap();
    let stake_smt = *matches.get_one::<bool>("stake-smt").unwrap();
    let delegate_smt = *matches.get_one::<bool>("delegate-smt").unwrap();
    let withdraw = *matches.get_one::<bool>("withdraw").unwrap();
    let reward = *matches.get_one::<bool>("reward").unwrap();

    let ckb = match net {
        "dev" => {
            println!("dev net");
            set_network_type(NetworkType::Devnet);
            CkbRpcClient::new("http://127.0.0.1:8114")
        }
        "test" => {
            println!("test net");
            set_network_type(NetworkType::Testnet);
            CkbRpcClient::new("https://testnet.ckb.dev")
        }
        "main" => {
            println!("main net");
            set_network_type(NetworkType::Mainnet);
            CkbRpcClient::new("https://mainnet.ckb.dev")
        }
        _ => unimplemented!(),
    };

    if faucet {
        faucet_tx(&ckb).await;
    }

    if init {
        init_tx(&ckb).await;
    }

    if mint {
        mint_tx(&ckb).await;
    }

    if stake.is_some() {
        match stake.unwrap().as_str() {
            "first" => first_stake_tx(&ckb).await,
            "add" => add_stake_tx(&ckb).await,
            "redeem" => reedem_stake_tx(&ckb).await,
            _ => unimplemented!(),
        }
    }

    if delegate.is_some() {
        match delegate.unwrap().as_str() {
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

    if withdraw {
        withdraw_tx(&ckb).await;
    }

    if reward {
        reward_tx(&ckb).await;
    }
}
