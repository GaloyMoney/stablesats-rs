use thiserror::Error;

#[derive(Error, Debug)]
pub enum QuoteError {
    #[error("QuotesError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("QuotesError - EntityError: {0}")]
    EntityError(#[from] crate::entity::EntityError),
    #[error("QuotesError - Quotes is already accepted")]
    QuoteAlreadyAccepted,
    #[error("QuotesError - Quote has expired")]
    QuoteExpiredError,
}
