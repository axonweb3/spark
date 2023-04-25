use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Create DB path {0}")]
    CreateDB(io::Error),
}
