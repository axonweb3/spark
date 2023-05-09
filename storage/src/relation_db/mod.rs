use anyhow::Result;
use async_trait::async_trait;
use migration::{Migrator, MigratorTrait};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, CursorTrait, Database, DbConn, EntityTrait, QueryFilter,
};

use crate::error::StorageError;
use common::traits::query::TransactionStorage;
use common::types::{
    relation_db::transaction::{self, Model},
    smt::Address,
};

pub async fn establish_connection(database_url: &str) -> Result<DbConn> {
    let db = Database::connect(database_url).await?;
    Migrator::up(&db, None).await?;

    Ok(db)
}

pub struct TransactionHistory {
    db: DbConn,
}

impl TransactionHistory {
    pub async fn new(database_url: &str) -> Self {
        let db = establish_connection(database_url).await.unwrap();
        Self { db }
    }
}

#[async_trait]
impl TransactionStorage for TransactionHistory {
    async fn insert(&mut self, tx_record: transaction::ActiveModel) -> Result<()> {
        let tx_record = tx_record.insert(&self.db).await?;
        log::info!(
            "Transaction created with address: {}, timestamp: {}, tx_hash: {}",
            tx_record.address,
            tx_record.timestamp,
            tx_record.tx_hash
        );
        Ok(())
    }

    async fn get_records_by_address(
        &self,
        addr: Address,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Model>> {
        let mut cursor = transaction::Entity::find()
            .filter(transaction::Column::Address.eq(addr.to_string()))
            .cursor_by(transaction::Column::Id);
        cursor.after(offset).before(offset + limit);
        match cursor.all(&self.db).await {
            Ok(records) => Ok(records),
            Err(e) => Err(StorageError::SqlCursorError(e).into()),
        }
    }
}
