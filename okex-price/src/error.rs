use serde_json::Error as SerdeError;
use thiserror::Error;
use tokio::sync::broadcast::error::SendError;
use tokio_tungstenite::tungstenite::error::Error as TungsteniteError;

use shared::{
    payload::*,
    pubsub::{Envelope, PublisherError},
};

#[allow(clippy::large_enum_variant)]
#[derive(Error, Debug)]
pub enum PriceFeedError {
    #[error("PriceFeedError - OkexWsError: {0}")]
    OkexWsError(#[from] TungsteniteError),
    #[error("PriceFeedError - EmptyPriceData: OkexPriceTick.data was empty")]
    EmptyPriceData,
    #[error("PriceFeedError - EmptyOrderBookData: OkexOrderBook.data was empty")]
    EmptyOrderBookData,
    #[error("PriceFeedError - EmptyBookSide: OkexOrderBook had empty book side")]
    EmptyBookSide,
    #[error("PriceFeedError - InvalidTimestamp: {0}")]
    InvalidTimestamp(#[from] shared::time::TimeStampError),
    #[error("PriceFeedError - SerdeError: {0}")]
    SerializationError(#[from] SerdeError),
    #[error("PriceFeedError - PublisherError: {0}")]
    PublisherError(#[from] PublisherError),
    #[error("PriceFeedError - PricePublish: {0}")]
    PricePublish(#[from] SendError<Envelope<PriceStreamPayload>>),
    #[error("PriceFeedError - OrderBookConversion: {0}")]
    OrderBookConversion(#[from] anyhow::Error),
    #[error("PriceFeedError - DepthValidation: {0}")]
    DepthValidation(String),
    #[error("PriceFeedError - InitialFullLoad: initial full load empty")]
    InitialFullLoad,
    #[error("PriceFeedError: CheckSumValidation - Can't validate accuracy of depth data")]
    CheckSumValidation,
    #[error("PriceFeedError: StreamEnded - Stream ended unexpectedly")]
    StreamEnded,
    #[error("PriceFeedError: StreamStalled - Stream ended unexpectedly")]
    StreamStalled,
}
