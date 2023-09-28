use thiserror::Error;

use crate::{currency::CurrencyError, order_book_cache::OrderBookCacheError};
use shared::{pubsub::SubscriberError, time::*};

#[allow(clippy::large_enum_variant)]
#[derive(Error, Debug)]
pub enum PriceAppError {
    #[error("PriceAppError - CurrencyError: {0}")]
    CurrencyError(#[from] CurrencyError),
    #[error("PriceAppError - SubscriberError: {0}")]
    SubscriberError(#[from] SubscriberError),
    #[error("PriceAppError - ExchangePriceCacheError: {0}")]
    ExchangePriceCacheError(#[from] ExchangePriceCacheError),
    #[error("PriceAppError - DecimalConversion: {0}")]
    DecimalConversion(#[from] rust_decimal::Error),
}

#[derive(Error, Debug)]
pub enum ExchangePriceCacheError {
    #[error("StalePrice: last update was at {0}")]
    StalePrice(TimeStamp),
    #[error("No price data available")]
    NoPriceAvailable,
    #[error("OrderBook: {0:?}")]
    OrderBookCache(#[from] OrderBookCacheError),
}
