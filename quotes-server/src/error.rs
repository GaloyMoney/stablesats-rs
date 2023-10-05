use thiserror::Error;

use crate::{price::ExchangePriceCacheError, quote::QuoteError};

#[derive(Error, Debug)]
pub enum QuotesAppError {
    #[error("QuotesAppError: {0}")]
    ExchangePriceCacheError(#[from] ExchangePriceCacheError),
    #[error("QuotesAppError: {0}")]
    QuoteError(#[from] QuoteError),
}
