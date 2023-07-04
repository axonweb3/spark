mod config;

use std::{env, sync::Arc};

use api::{run_server, DefaultAPIAdapter};
use config::SparkConfig;
use rpc_client::ckb_client::ckb_rpc_client::CkbRpcClient;
use storage::{SmtManager, TransactionHistory};
use tx_builder::init_static_variables;

#[tokio::main]
async fn main() {
    let args = env::args().nth(1).expect("Missing env variable");
    let config: SparkConfig = config::parse_file(args).expect("Failed to parse config file");
    init_static_variables(
        config.network_type,
        config.metadata_type_id,
        config.checkpoint_type_id,
    );

    let ckb_rpc_client = Arc::new(CkbRpcClient::new(&config.ckb_node_url));
    let rdb = Arc::new(TransactionHistory::new(&config.rdb_url).await);
    let kvdb = Arc::new(SmtManager::new(&config.kvdb_path));

    let api_adapter = Arc::new(DefaultAPIAdapter::new(ckb_rpc_client, rdb, kvdb));
    let _handle = run_server(api_adapter, config.rpc_listen_address)
        .await
        .unwrap();

    println!("Hello, world!");
}
