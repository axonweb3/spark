// use ethereum_types::H256;
// use sea_orm::entity::prelude::*;

// use super::smt::Address;

// /// Fruit entity
// #[derive(Copy, Clone, Default, Debug, DeriveEntity)]
// pub struct Entity;

// impl EntityName for Entity {
//     fn table_name(&self) -> &str {
//         "transaction_history"
//     }
// }

// /// Fruit model
// #[derive(Clone, Debug, PartialEq, Eq, DeriveModel, DeriveActiveModel)]
// pub struct Model {
//     /// id field
//     pub id: i32,
//     /// name field
//     pub address: String,
//     pub timestamp: u8,
//     pub operation: OperationType,
//     pub tx_hash: String,
//     pub amount: u128,
//     pub status: TransactionStatus
// }

// #[derive(PartialEq, Eq)]
// pub enum TransactionStatus {
//     Success,
//     Pending,
//     Fail,
// }

// #[derive(PartialEq, Eq)]
// pub enum OperationType {
//     StakeAdd,
//     StakeRedeem,
//     DelegateAdd,
//     DelegateRedeem,
//     Withdraw,
// }

// pub struct TransactionRecord {
//     address:   Address,
//     timestamp: u8,
//     operation: OperationType,
//     tx_hash:   H256,
//     amount:    u128,
//     status:    TransactionStatus,
// }

// /// Fruit column
// #[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
// pub enum Column {
//     /// Id column
//     Id,
//     Address,
//     Timestamp,
//     Operation,
//     TxHash,
//     Amount,
//     Status
// }

// /// Fruit primary key
// #[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
// pub enum PrimaryKey {
//     /// Id primary key
//     Id,
// }

// impl PrimaryKeyTrait for PrimaryKey {
//     type ValueType = i32;

//     fn auto_increment() -> bool {
//         true
//     }
// }

// impl ColumnTrait for Column {
//     type EntityName = Entity;

//     fn def(&self) -> ColumnDef {
//         match self {
//             Self::Id => ColumnType::Integer.def(),
//             Self::Address => ColumnType::String(42).def(),
//             Self::Amount => ColumnType::BigUnsigned.def(),
//             Self::Operation => ColumnType::Enum { name: "OperationType",
// variants: () }.def()

//         }
//     }
// }

// impl ActiveModelBehavior for ActiveModel {}
