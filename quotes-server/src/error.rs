use thiserror::Error;

use crate::{price::ExchangePriceCacheError, quote::QuoteError};

#[derive(Error, Debug)]
pub enum QuotesAppError {
    #[error("QuotesAppError: {0}")]
    ExchangePriceCacheError(#[from] ExchangePriceCacheError),
    #[error("QuotesAppError: {0}")]
    QuoteError(#[from] QuoteError),
    #[error("{0}")]
    LedgerError(#[from] ledger::LedgerError),
    #[error("{0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("QuotesAppError: Quote already accepted for id: {0}")]
    QuoteAlreadyAccepted(String),
    #[error("QuotesAppError: Quote expired for id: {0}")]
    QuoteExpired(String),
}
