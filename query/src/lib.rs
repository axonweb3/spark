mod adapter;
mod temp_mock;
mod tests;

use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum QueryError {
    #[error("data store disconnected")]
    Disconnect(#[from] io::Error),
    #[error("http server error {0}")]
    HttpServer(String),
    #[error("invalid method (expected {expected:?}, found {found:?})")]
    InvalidMethod { expected: String, found: String },
    #[error("unknown query error")]
    Unknown,
}
