use serde_json::Error as SerdeError;
use thiserror::Error;
use tokio_tungstenite::tungstenite::error::Error as TungsteniteError;

use shared::pubsub::PublisherError;

#[derive(Error, Debug)]
pub enum PriceFeedError {
    #[error("Pricefeed connection to websocket failed: {0}")]
    OkexWsError(#[from] TungsteniteError),
    #[error("OkexPriceTick.data was empty")]
    EmptyPriceData,
    #[error("Couldn't parse timestamp")]
    InvalidTimestamp(#[from] std::num::ParseIntError),
    #[error("Pricefeed encountered a serialization error: {0}")]
    SerializationError(#[from] SerdeError),
    #[error("{0}")]
    PublisherError(#[from] PublisherError),
}
