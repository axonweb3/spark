mod config;

use std::{env, sync::Arc};

use api::{run_server, DefaultAPIAdapter};
use config::SparkConfig;
use storage::{SmtManager, TransactionHistory};

#[tokio::main]
async fn main() {
    let args = env::args().nth(1).expect("Missing env variable");
    let config: SparkConfig = config::parse_file(args).expect("Failed to parse config file");

    let rdb = Arc::new(TransactionHistory::new(&config.rdb_url).await);
    let kvdb = Arc::new(SmtManager::new(&config.kvdb_path));
    let api_adapter = Arc::new(DefaultAPIAdapter::new(rdb, kvdb));
    let _handle = run_server(api_adapter, config.rpc_listen_address)
        .await
        .unwrap();

    println!("Hello, world!");
}
