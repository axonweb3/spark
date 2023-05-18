use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RpcError {
    #[error("RocksDB creation error {0}")]
    ConnectionAborted(io::Error),

    #[error("jsonrpc output failure {0}")]
    InvalidData(io::Error),
}
