use thiserror::Error;

use crate::exchange_price_cache::ExchangePriceCacheError;
use shared::{currency::CurrencyError, pubsub::SubscriberError};

#[derive(Error, Debug)]
pub enum PriceAppError {
    #[error("No price data available")]
    NoPriceDataAvailable,
    #[error("Price data is stale")]
    StalePriceData,
    #[error("{0}")]
    CurrencyError(#[from] CurrencyError),
    #[error("{0}")]
    SubscriberError(#[from] SubscriberError),
    #[error("{0}")]
    ExchnagePriceCacheError(#[from] ExchangePriceCacheError),
}
