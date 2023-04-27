mod tests;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum QueryError {
    // #[error("data store disconnected")]
    // Disconnect(#[from] io::Error),
    #[error("http server error {0}")]
    HttpServer(String),
    // #[error("invalid header (expected {expected:?}, found {found:?})")]
    // InvalidHeader {
    //     expected: String,
    //     found: String,
    // },
    // #[error("unknown data store error")]
    // Unknown,
}