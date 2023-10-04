use thiserror::Error;

#[derive(Error, Debug)]
pub enum QuoteError {
    #[error("WalletError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("WalletError - EntityError: {0}")]
    EntityError(#[from] crate::entity::EntityError),
}
