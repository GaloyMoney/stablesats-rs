use thiserror::Error;

use crate::app::PriceAppError;

#[allow(clippy::large_enum_variant)]
#[derive(Error, Debug)]
pub enum PriceServerError {
    #[error("PriceServerError: {0}")]
    TonicError(#[from] tonic::transport::Error),
    #[error("PriceServerError: {0}")]
    AppError(#[from] PriceAppError),
}
