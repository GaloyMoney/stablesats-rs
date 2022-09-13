use thiserror::Error;

use crate::app::PriceAppError;

#[allow(clippy::large_enum_variant)]
#[derive(Error, Debug)]
pub enum PriceServerError {
    #[error("PriceServerError - TonicError: {0}")]
    TonicError(#[from] tonic::transport::Error),
    #[error("PriceServerError - AppError: {0}")]
    AppError(#[from] PriceAppError),
}
