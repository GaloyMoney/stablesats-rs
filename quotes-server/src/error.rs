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
    #[error("QuotesServerError - CouldNotParseIncomingUuid: {0}")]
    CouldNotParseIncomingUuid(uuid::Error),
}
