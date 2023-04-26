use async_trait::async_trait;
use migration::{DbErr, Migrator, MigratorTrait};
use sea_orm::sea_query::error::Error;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, CursorTrait, Database, DbConn, EntityTrait, QueryFilter,
};
use transaction_entity::transaction::{self, Model};

use crate::traits::relation_db::TransactionStorage;
use crate::types::smt::Address;

pub async fn establish_connection() -> Result<DbConn, Error> {
    let database_url = std::env::var("DATABASE_URL").unwrap();
    let db = Database::connect(&database_url)
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
    pub async fn new() -> Result<Self, Error> {
        let db = establish_connection().await?;
        Ok(Self { db })
    }
}

#[async_trait]
impl TransactionStorage for TransactionHistory {
    async fn insert(&mut self, tx_record: transaction::ActiveModel) -> Result<(), DbErr> {
        let tx_record: transaction::Model = tx_record.insert(&self.db).await?;
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
    ) -> Result<Vec<Model>, DbErr> {
        let mut cursor = transaction::Entity::find()
            .filter(transaction::Column::Address.eq(addr.to_string()))
            .cursor_by(transaction::Column::Id);
        cursor.after(offset).before(offset + limit);
        cursor.all(&self.db).await
    }
}
