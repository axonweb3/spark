use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .create_table(
                Table::create()
                    .table(Transaction::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Transaction::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Transaction::Address)
                            .string_len(42)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Transaction::Timestamp)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Transaction::Operation).integer().not_null())
                    .col(ColumnDef::new(Transaction::Event).integer().not_null())
                    .col(
                        ColumnDef::new(Transaction::TxHash)
                            .string_len(66)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Transaction::TotalAmount)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Transaction::StakeAmount)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Transaction::DelegateAmount)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Transaction::WithdrawableAmount)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Transaction::StakeRate)
                            .string_len(10)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Transaction::DelegateRate)
                            .string_len(10)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Transaction::Epoch).integer().not_null())
                    .col(ColumnDef::new(Transaction::Status).integer().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .drop_table(Table::drop().table(Transaction::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Transaction {
    Table,
    Id,
    Address,
    Timestamp,
    Operation,
    Event,
    TxHash,
    TotalAmount,
    StakeAmount,
    DelegateAmount,
    WithdrawableAmount,
    StakeRate,
    DelegateRate,
    Epoch,
    Status,
}
