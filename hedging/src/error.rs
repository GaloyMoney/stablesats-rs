use thiserror::Error;

use shared::{
    pubsub::{PublisherError, SubscriberError},
    sqlxmq::JobExecutionError,
};

#[allow(clippy::large_enum_variant)]
#[derive(Error, Debug)]
pub enum HedgingError {
    #[error("HedgingError - Subscriber: {0}")]
    Subscriber(#[from] SubscriberError),
    #[error("HedgingError - Publisher: {0}")]
    Publisher(#[from] PublisherError),
    #[error("HedgingError - SerdeJson: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("HedgingError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("HedgingError - Migrate: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error("HedgingError - OkxClient: {0}")]
    OkxClient(#[from] okx_client::OkxClientError),
    #[error("HedgingError - GaloyClient: {0}")]
    GaloyClient(#[from] galoy_client::GaloyClientError),
    #[error("HedgingError - BitfinexClient: {0}")]
    BitfinextClient(#[from] bitfinex_client::BitfinexClientError),
    #[error("HedgingError - NoJobDataPresent")]
    NoJobDataPresent,
    #[error("UserTradesError - Leger: {0}")]
    Ledger(#[from] ledger::LedgerError),
}

impl JobExecutionError for HedgingError {}
