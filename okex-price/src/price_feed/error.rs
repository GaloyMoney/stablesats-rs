use serde_json::Error as SerdeError;
use thiserror::Error;
use tokio_tungstenite::tungstenite::error::Error as TungsteniteError;

#[derive(Error, Debug)]
pub enum PriceFeedError {
    #[error("Pricefeed connection to websocket failed: {0}")]
    ConnectionError(#[from] TungsteniteError),
    #[error("Pricefeed encountered a serialization error: {0}")]
    SerializationError(#[from] SerdeError),
}
