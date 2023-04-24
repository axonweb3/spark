use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Create DB path {0}")]
    CreateDB(io::Error),

    #[error("rocksdb {0}")]
    RocksDB(#[from] rocksdb::Error),

    #[error("Invalid block number: {0}")]
    InvalidBlockNumber(u64),

    #[error("Update block number error: {e:?}, number: {number:?}")]
    UpdateBlockNumber { e: String, number: u64 },

    #[error("Get block number error: {0}")]
    GetBlockNumber(String),

    #[error("Decode block number failed: {0}")]
    DecodeBlockNumber(rlp::DecoderError),

}
