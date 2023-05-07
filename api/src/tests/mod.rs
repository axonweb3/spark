#[cfg(test)]
mod tests {
    use crate::{adapter::DefaultAPIAdapter, jsonrpc::mock_server};
    use common::{
        traits::query::TransactionStorage,
        types::{relation_db::transaction, H160},
        AnyError, Result,
    };
    use jsonrpsee::http_client::HttpClientBuilder;
    use std::{path::PathBuf, sync::Arc};
    use storage::{
        relation_db::{establish_connection, Set, TransactionHistory},
        smt::SmtManager,
    };

    static RELATIONDB_URL: &str = "sqlite::memory:";
    static ROCKSDB_PATH: &str = "./free-space/smt";

    pub async fn mock_data(
        hash: String,
        amount: String,
    ) -> Result<transaction::ActiveModel, AnyError> {
        Ok(transaction::ActiveModel {
            address: Set(H160::zero().to_string().to_owned()),
            timestamp: Set(1),
            operation: Set(1),
            event: Set(1),
            tx_hash: Set(hash.to_owned()),
            amount: Set(amount.to_owned()),
            status: Set(1),
            epoch: Set(1),
            ..Default::default()
        })
    }

    async fn _mock_adapter() {
        let db = establish_connection(RELATIONDB_URL).await.unwrap();
        let relation_db = TransactionHistory { db };
        let mut smt_path = PathBuf::from(ROCKSDB_PATH);
        smt_path.push("stake");
        let smt_manager = SmtManager::new(smt_path);
        let _adapter = DefaultAPIAdapter::new(Arc::new(relation_db), Arc::new(smt_manager));
    }

    #[tokio::test]
    async fn mock_db() {
        let mut relation_db1 = TransactionHistory::new(RELATIONDB_URL).await;
        let data0 = mock_data("0x01".to_owned(), "100".to_owned())
            .await
            .unwrap();
        let data1 = mock_data("0x02".to_owned(), "100".to_owned())
            .await
            .unwrap();
        let _ = relation_db1.insert(data0).await.unwrap();
        let _ = relation_db1.insert(data1).await.unwrap();
        let res = relation_db1
            .get_records_by_address(H160::zero(), 0, 4)
            .await;
        println!("{:?}", res);
    }

    #[tokio::test]
    async fn mock_jsonrpc_server() -> Result<()> {
        let db = establish_connection(RELATIONDB_URL).await?;
        let relation_db = TransactionHistory { db };
        let mut smt_path = PathBuf::from(ROCKSDB_PATH);
        smt_path.push("stake");
        let smt_manager = SmtManager::new(smt_path);
        let adapter = DefaultAPIAdapter::new(Arc::new(relation_db), Arc::new(smt_manager));
        let server_addr = mock_server(Arc::new(adapter)).await?;
        let url = format!("http://{:?}", server_addr);
        // println!("addr: {:?}", url);
        let _client = HttpClientBuilder::default().build(url).unwrap();

        Ok(())
    }
}
