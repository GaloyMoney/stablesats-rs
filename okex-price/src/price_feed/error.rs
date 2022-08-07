use serde_json::Error as SerdeError;
use thiserror::Error;
use tokio_tungstenite::tungstenite::error::Error as TungsteniteError;

use shared::pubsub::PublisherError;

#[derive(Error, Debug)]
pub enum PriceFeedError {
    #[error("PriceFeedError: {0}")]
    OkexWsError(#[from] TungsteniteError),
    #[error("PriceFeedError: OkexPriceTick.data was empty")]
    EmptyPriceData,
    #[error("PriceFeedError: {0}")]
    InvalidTimestamp(#[from] std::num::ParseIntError),
    #[error("PriceFeedError: {0}")]
    SerializationError(#[from] SerdeError),
    #[error("PriceFeedError: {0}")]
    PublisherError(#[from] PublisherError),
}
