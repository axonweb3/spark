use crate::types::transaction::{self, Model};
use anyhow::Result;
use async_trait::async_trait;
use migration::{Migrator, MigratorTrait};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, CursorTrait, Database, DbConn, EntityTrait, QueryFilter,
};

use crate::error::StorageError;
use crate::traits::relation_db::TransactionStorage;
use crate::types::smt::Address;

pub async fn establish_connection(database_url: &str) -> Result<DbConn> {
    let db = Database::connect(database_url)
        .await
        .expect("Failed to setup the database");
    Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations for tests");

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
        println!(
            "Post created with ID: {}, TITLE: {}",
            tx_record.id, tx_record.address
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
