pub mod traits;
pub mod types;
use std::error::Error as Err;
use thiserror::Error;
use std::io;

#[derive(Error, Debug)]
pub enum TempQueryError {
    #[error("data store disconnected")]
    Disconnect(#[from] io::Error),
    #[error("http server error {0}")]
    HttpServer(String),
    #[error("invalid method (expected {expected:?}, found {found:?})")]
    InvalidMethod { expected: String, found: String },
    #[error("unknown query error")]
    Unknown,
    #[error(transparent)]
    Other(#[from] anyhow::Error), 
}

impl From<TempQueryError> for Box<dyn Err + Send> {
    fn from(error: TempQueryError) -> Self {
        Box::new(error) as Box<dyn Err + Send>
    }
}