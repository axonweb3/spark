use std::sync::{Arc, Mutex};

use common::logger;
use common::types::tx_builder::NetworkType;
use config::types::PrivKeys;
use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;
use storage::SmtManager;
use tx_builder::ckb::helper::OmniEth;
use tx_builder::set_network_type;

use crate::config::{parse_log_config, parse_priv_keys};
use crate::helper::smt::create_smt;
use crate::tx::*;

mod cases;
mod config;
mod helper;
mod mock;
mod tx;

pub const PRIV_KEYS_PATH: &str = "./src/config/priv_keys.toml";
pub const TYPE_IDS_PATH: &str = "./src/config/type_ids.toml";
pub const LOG_CONFIG_PATH: &str = "./src/config/log.toml";
pub const ROCKSDB_PATH: &str = "./free-space/smt";
pub const MAX_TRY: u64 = 1000;

lazy_static::lazy_static! {
    pub static ref SMT: Arc<Mutex<SmtManager>> = Arc::new(Mutex::new(SmtManager::new(ROCKSDB_PATH)));
}

#[tokio::main]
async fn main() {
    let cmd = clap::Command::new("spark")
        .version(clap::crate_version!())
        .subcommand_required(true)
        .subcommand(
            clap::Command::new("cases")
                .about("Test cases")
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
                    clap::Arg::new("all")
                        .long("all")
                        .required(false)
                        .num_args(0)
                        .help("Test all tx"),
                )
                .arg(
                    clap::Arg::new("delegate")
                        .long("delegate")
                        .required(false)
                        .num_args(0)
                        .help("Test delegate tx"),
                )
                .arg(
                    clap::Arg::new("delegate-smt")
                        .long("delegate-smt")
                        .required(false)
                        .num_args(0)
                        .help("Test delegate smt tx"),
                )
                .arg(
                    clap::Arg::new("stake")
                        .long("stake")
                        .required(false)
                        .num_args(0)
                        .help("Test stake tx"),
                )
                .arg(
                    clap::Arg::new("stake-smt")
                        .long("stake-smt")
                        .required(false)
                        .num_args(0)
                        .help("Test stake smt tx"),
                )
                .arg(
                    clap::Arg::new("metadata")
                        .long("metadata")
                        .required(false)
                        .num_args(0)
                        .help("Test metadata tx"),
                )
                .arg(
                    clap::Arg::new("reward")
                        .long("reward")
                        .required(false)
                        .num_args(0)
                        .help("Test reward tx"),
                )
                .arg(
                    clap::Arg::new("withdraw")
                        .long("withdraw")
                        .required(false)
                        .num_args(0)
                        .help("Test withdraw tx"),
                ),
        )
        .subcommand(
            clap::Command::new("tx")
                .about("Test single tx")
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
                    clap::Arg::new("metadata")
                        .short('a')
                        .required(false)
                        .num_args(0)
                        .help("Test metadata tx"),
                )
                .arg(
                    clap::Arg::new("reward")
                        .short('r')
                        .required(false)
                        .num_args(0)
                        .help("Test reward tx"),
                ),
        )
        .subcommand(
            clap::Command::new("users")
                .about("Show users information")
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
                    clap::Arg::new("address")
                        .short('a')
                        .long("address")
                        .required(false)
                        .num_args(0)
                        .help("Show users address"),
                ),
        );

    register_log();

    let priv_keys = parse_priv_keys(PRIV_KEYS_PATH);

    let matches = cmd.get_matches();
    match matches.subcommand() {
        Some(("cases", matches)) => run_test_cases(matches, priv_keys).await,
        Some(("tx", matches)) => run_single_tx(matches, priv_keys).await,
        Some(("users", matches)) => view_users(matches, priv_keys),
        _ => unimplemented!(),
    }
}

async fn run_test_cases(matches: &clap::ArgMatches, priv_keys: PrivKeys) {
    let net = matches.get_one::<String>("net").unwrap().as_str();
    let all = matches.get_one::<bool>("all").unwrap();
    let delegate = matches.get_one::<bool>("delegate").unwrap();
    let delegate_smt = matches.get_one::<bool>("delegate-smt").unwrap();
    let stake = matches.get_one::<bool>("stake").unwrap();
    let stake_smt = matches.get_one::<bool>("stake-smt").unwrap();
    let metadata = matches.get_one::<bool>("metadata").unwrap();
    let reward = matches.get_one::<bool>("reward").unwrap();
    let withdraw = matches.get_one::<bool>("withdraw").unwrap();

    let ckb = parse_ckb_net(net);
    let smt = create_smt();

    if *all {
        cases::all::run_all_tx(&ckb, &smt, priv_keys.clone()).await;
    }

    if *delegate {
        cases::delegate::run_delegate_case(&ckb, &smt, priv_keys.clone()).await;
    }

    if *delegate_smt {
        cases::delegate_smt::run_delegate_smt_case(&ckb, &smt, priv_keys.clone()).await;
    }

    if *stake {
        cases::stake::run_stake_case(&ckb, &smt, priv_keys.clone()).await;
    }

    if *stake_smt {
        cases::stake_smt::run_stake_smt_case(&ckb, &smt, priv_keys.clone()).await;
    }

    if *metadata {
        cases::metadata::run_metadata_case(&ckb, &smt, priv_keys.clone()).await;
    }

    if *reward {
        cases::reward::run_reward_case(&ckb, &smt, priv_keys.clone()).await;
    }

    if *withdraw {
        cases::withdraw::run_withdraw_case(&ckb, &smt, priv_keys.clone()).await;
    }
}

async fn run_single_tx(matches: &clap::ArgMatches, priv_keys: PrivKeys) {
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
    let metadata = *matches.get_one::<bool>("metadata").unwrap();
    let reward = *matches.get_one::<bool>("reward").unwrap();

    let ckb = parse_ckb_net(net);
    let smt = create_smt();

    let seeder_key = priv_keys.seeder_privkey.clone().into_h256().unwrap();
    let kicker_key = priv_keys.staker_privkeys[0].clone().into_h256().unwrap();
    let staker_key = priv_keys.staker_privkeys[0].clone().into_h256().unwrap();
    let delegator_key = priv_keys.delegator_privkeys[0].clone().into_h256().unwrap();
    let staker_eth_addr = OmniEth::new(staker_key.clone()).address().unwrap();

    if staker_key == delegator_key {
        panic!("Stakers can't delegate themselves.");
    }

    if faucet {
        run_faucet_tx(&ckb, priv_keys.clone()).await;
    } else if init {
        run_init_tx(&ckb, seeder_key, vec![staker_key.clone()], 10).await;
    } else if mint {
        run_mint_tx(&ckb, priv_keys.clone()).await;
    } else if stake.is_some() {
        match stake.unwrap().as_str() {
            "first" => first_stake_tx(&ckb, staker_key, 100).await,
            "add" => add_stake_tx(&ckb, staker_key, 10, 0).await.unwrap(),
            "redeem" => redeem_stake_tx(&ckb, staker_key, 10, 0).await.unwrap(),
            _ => unimplemented!(),
        }
    } else if delegate.is_some() {
        match delegate.unwrap().as_str() {
            "first" => first_delegate_tx(&ckb, delegator_key, staker_eth_addr)
                .await
                .unwrap(),
            "add" => add_delegate_tx(&ckb, delegator_key, staker_eth_addr, 10, 0)
                .await
                .unwrap(),
            "redeem" => redeem_delegate_tx(&ckb, delegator_key, staker_eth_addr, 10, 0)
                .await
                .unwrap(),
            _ => unimplemented!(),
        }
    } else if checkpoint {
        run_checkpoint_tx(&ckb, kicker_key.clone(), vec![staker_key.clone()], 1).await;
    } else if stake_smt {
        stake_smt_tx(&ckb, &smt, kicker_key, vec![staker_key.clone()], 0).await;
    } else if delegate_smt {
        delegate_smt_tx(&ckb, &smt, kicker_key, vec![delegator_key], 0).await;
    } else if withdraw {
        let user_key = priv_keys.staker_privkeys[0].clone().into_h256().unwrap();
        run_withdraw_tx(&ckb, user_key, 2).await;
    } else if metadata {
        run_metadata_tx(&ckb, &smt, kicker_key, 0).await;
    } else if reward {
        let user_key = priv_keys.staker_privkeys[0].clone().into_h256().unwrap();
        run_reward_tx(&ckb, &smt, user_key, 4).await.unwrap();
    } else {
        unimplemented!();
    }
}

fn view_users(matches: &clap::ArgMatches, priv_keys: PrivKeys) {
    let net = matches.get_one::<String>("net").unwrap().as_str();
    let address = matches.get_one::<bool>("address").unwrap();

    parse_ckb_net(net);

    if *address {
        let seeder_key = priv_keys.seeder_privkey.into_h256().unwrap();
        let omni_eth = OmniEth::new(seeder_key);
        println!(
            "seeder ckb addres: {}, eth address: {}",
            omni_eth.ckb_address().unwrap(),
            omni_eth.address().unwrap(),
        );

        for (i, staker_privkey) in priv_keys.staker_privkeys.into_iter().enumerate() {
            let privkey = staker_privkey.clone().into_h256().unwrap();
            let omni_eth = OmniEth::new(privkey);
            println!(
                "staker{} ckb addres: {}, eth address: {}",
                i,
                omni_eth.ckb_address().unwrap(),
                omni_eth.address().unwrap(),
            );
        }

        for (i, delegator_privkey) in priv_keys.delegator_privkeys.into_iter().enumerate() {
            let privkey = delegator_privkey.clone().into_h256().unwrap();
            let omni_eth = OmniEth::new(privkey);
            println!(
                "delegator{} ckb addres: {}, eth address: {}",
                i,
                omni_eth.ckb_address().unwrap(),
                omni_eth.address().unwrap(),
            );
        }
    }
}

fn register_log() {
    let config = parse_log_config(LOG_CONFIG_PATH);

    logger::init(
        config.filter.clone(),
        config.log_to_console,
        config.console_show_file_and_line,
        config.log_to_file,
        config.log_path.clone(),
        config.file_size_limit,
    );
}

fn parse_ckb_net(net: &str) -> CkbRpcClient {
    match net {
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
    }
}
