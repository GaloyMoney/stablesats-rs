use thiserror::Error;

use crate::price::ExchangePriceCacheError;

#[derive(Error, Debug)]
pub enum QuotesAppError {
    #[error("QuotesAppError: {0}")]
    ExchangePriceCacheError(#[from] ExchangePriceCacheError),
}
