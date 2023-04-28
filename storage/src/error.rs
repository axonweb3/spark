use migration::DbErr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Sql cursor error {0}")]
    SqlCursorError(DbErr),
}
