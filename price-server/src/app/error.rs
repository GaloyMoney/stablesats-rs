use thiserror::Error;

use crate::{currency::CurrencyError, exchange_price_cache::ExchangePriceCacheError};
use shared::pubsub::SubscriberError;

#[allow(clippy::large_enum_variant)]
#[derive(Error, Debug)]
pub enum PriceAppError {
    #[error("PriceAppError - CurrencyError: {0}")]
    CurrencyError(#[from] CurrencyError),
    #[error("PriceAppError - SubscriberError: {0}")]
    SubscriberError(#[from] SubscriberError),
    #[error("PriceAppError - ExchangePriceCacheError: {0}")]
    ExchangePriceCacheError(#[from] ExchangePriceCacheError),
    #[error("PriceAppError - StdDurationConversionError: {0}")]
    StdDurationConversionError(#[from] chrono::OutOfRangeError),
}
