use std::sync::Arc;

use api::{run_server, DefaultAPIAdapter};
use storage::{SmtManager, TransactionHistory};

const RDB_PATH: &str = "";
const KV_PATH: &str = "./free-space/kvdb";

#[tokio::main]
async fn main() {
    let rdb = Arc::new(TransactionHistory::new(RDB_PATH).await);
    let kvdb = Arc::new(SmtManager::new(KV_PATH));
    let api_adapter = Arc::new(DefaultAPIAdapter::new(rdb, kvdb));
    let _handle = run_server(api_adapter).await.unwrap();

    println!("Hello, world!");
}
