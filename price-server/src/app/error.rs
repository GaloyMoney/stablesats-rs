use thiserror::Error;

use crate::{currency::CurrencyError, error::ExchangePriceCacheError};
use shared::pubsub::SubscriberError;

#[allow(clippy::large_enum_variant)]
#[derive(Error, Debug)]
pub enum PriceAppError {
    #[error("PriceAppError - CurrencyError: {0}")]
    CurrencyError(#[from] CurrencyError),
    #[error("PriceAppError - SubscriberError: {0}")]
    SubscriberError(#[from] SubscriberError),
    #[error("PriceAppError - PriceTickCacheError: {0}")]
    ExchangePriceCacheError(#[from] ExchangePriceCacheError),
    #[error("PriceAppError - FloatingPointConversion: {0}")]
    FloatingPointConversion(#[from] rust_decimal::Error),
}
