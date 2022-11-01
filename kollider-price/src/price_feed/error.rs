use serde_json::Error as SerdeError;
use shared::pubsub::PublisherError;
use thiserror::Error;

use tokio_tungstenite::tungstenite::error::Error as TungsteniteError;

#[derive(Error, Debug)]
pub enum KolliderPriceFeedError {
    #[error("PriceFeedError - SerdeError: {0}")]
    SerializationError(#[from] SerdeError),

    #[error("PriceFeedError - PublisherError: {0}")]
    PublisherError(#[from] PublisherError),

    #[error("PriceFeedError - TungsteniteError: {0}")]
    TungsteniteError(#[from] TungsteniteError),
}
