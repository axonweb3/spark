use std::io;

use migration::DbErr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("RocksDB creation error {0}")]
    RocksDBCreationError(io::Error),

    #[error("Sql cursor error {0}")]
    SqlCursorError(DbErr),
}
