use shared::pubsub::PublisherError;
use thiserror::Error;
use tokio_tungstenite::tungstenite::error::Error as TungsteniteError;

#[derive(Error, Debug)]
pub enum OrderBookError {
    #[error("OrderBookError - OkexWsError: {0}")]
    OkexWsError(#[from] TungsteniteError),
    #[error("OrderBookError - EmptyOrderBook")]
    EmptyOrderBook,
    #[error("OrderBookError - InvalidTimestamp: {0}")]
    InvalidTimestamp(#[from] std::num::ParseIntError),
    #[error("OrderBookError - PublisherError: {0}")]
    PublisherError(#[from] PublisherError),
}
