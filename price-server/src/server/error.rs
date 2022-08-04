use thiserror::Error;

use crate::app::PriceAppError;

#[allow(clippy::large_enum_variant)]
#[derive(Error, Debug)]
pub enum PriceServerError {
    #[error("{0}")]
    TonicError(#[from] tonic::transport::Error),
    #[error("{0}")]
    AppError(#[from] PriceAppError),
}
