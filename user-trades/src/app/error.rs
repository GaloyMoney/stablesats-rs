use thiserror::Error;

use shared::pubsub::PublisherError;

use super::UserTradeBalancesError;

#[allow(clippy::large_enum_variant)]
#[derive(Error, Debug)]
pub enum UserTradesAppError {
    #[error("UserTradesAppError: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("UserTradesAppError: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error("UserTradesAppError: {0}")]
    UserTradeBalances(#[from] UserTradeBalancesError),
    #[error("UserTradesAppError: {0}")]
    PubSub(#[from] PublisherError),
}
