use thiserror::Error;

use shared::{pubsub::PublisherError, sqlxmq::JobExecutionError};

#[allow(clippy::large_enum_variant)]
#[derive(Error, Debug)]
pub enum UserTradesError {
    #[error("UserTradesError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("UserTradesError - Migrate: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error("UserTradesError - Publisher: {0}")]
    PubSub(#[from] PublisherError),
    #[error("UserTradesError - GaloyClient: {0}")]
    GaloyClient(#[from] galoy_client::GaloyClientError),
}

impl JobExecutionError for UserTradesError {}
