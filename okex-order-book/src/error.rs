use thiserror::Error;
use tokio_tungstenite::tungstenite::error::Error as TungsteniteError;

#[derive(Error, Debug)]
pub enum OrderBookError {
    #[error("PriceFeedError - OkexWsError: {0}")]
    OkexWsError(#[from] TungsteniteError),
}