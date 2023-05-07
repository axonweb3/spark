use common::{thiserror, AnyError, Error};
pub use jsonrpsee::core::Error as RpcError;
use jsonrpsee::types::ErrorObject;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("adapter error {0}")]
    Adapter(String),
    #[error("http server error {0}")]
    HttpServer(String),
    #[error("invalid method (expected {expected:?}, found {found:?})")]
    InvalidMethod { expected: String, found: String },
    #[error(transparent)]
    Other(#[from] AnyError),
}

impl<'a> From<ApiError> for ErrorObject<'a> {
    fn from(error: ApiError) -> Self {
        ErrorObject::owned(-32603, "Api error", Some(error.to_string()))
    }
}
