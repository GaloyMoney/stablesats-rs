use thiserror::Error;

#[derive(Error, Debug)]
pub enum UserTradesError {
    #[error("UserTradesError: {0}")]
    SqlxError(#[from] sqlx::Error),
}
