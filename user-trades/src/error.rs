use thiserror::Error;

use shared::pubsub::PublisherError;

#[allow(clippy::large_enum_variant)]
#[derive(Error, Debug)]
pub enum UserTradesError {
    #[error("UserTradesError: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("UserTradesError: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error("UserTradesError: {0}")]
    PubSub(#[from] PublisherError),
    #[error("UserTradesError: {0}")]
    GaloyClient(#[from] galoy_client::GaloyClientError),
    #[error("UserTradesError: {0}")]
    Conversion(String),
    #[error("UserTradesError: {0}")]
    Unify(String),
}
