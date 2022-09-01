use thiserror::Error;

use shared::pubsub::SubscriberError;

#[derive(Error, Debug)]
pub enum HedgingError {
    #[error("HedgingError: {0}")]
    PubSub(#[from] SubscriberError),
    #[error("HedgingError: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("HedgingError: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error("HedgingError: {0}")]
    Job(String)
}
