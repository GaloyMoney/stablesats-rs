use thiserror::Error;

use crate::error::QuotesAppError;

#[allow(clippy::large_enum_variant)]
#[derive(Error, Debug)]
pub enum QuotesServerError {
    #[error("QuotesServerError - TonicError: {0}")]
    TonicError(#[from] tonic::transport::Error),
    #[error("QuotesServerError - AppError: {0}")]
    AppError(#[from] QuotesAppError),
}
