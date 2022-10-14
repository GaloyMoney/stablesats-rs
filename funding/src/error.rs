use thiserror::Error;

use shared::{
    pubsub::{PublisherError, SubscriberError},
    sqlxmq::JobExecutionError,
};

#[allow(clippy::large_enum_variant)]
#[derive(Error, Debug)]
pub enum FundingError {
    #[error("FundingError - Subscriber: {0}")]
    Subscriber(#[from] SubscriberError),
    #[error("FundingError - Publisher: {0}")]
    Publisher(#[from] PublisherError),
    #[error("FundingError - SerdeJson: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("FundingError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("FundingError - Migrate: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error("FundingError - OkexClient: {0}")]
    OkexClient(#[from] okex_client::OkexClientError),
    #[error("FundingError - GaloyClient: {0}")]
    GaloyClient(#[from] galoy_client::GaloyClientError),
}

impl JobExecutionError for FundingError {}
