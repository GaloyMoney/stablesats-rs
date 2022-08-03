use thiserror::Error;

use crate::exchange_price_cache::ExchangePriceCacheError;
use shared::{currency::CurrencyError, pubsub::SubscriberError};

#[allow(clippy::large_enum_variant)]
#[derive(Error, Debug)]
pub enum PriceAppError {
    #[error("{0}")]
    CurrencyError(#[from] CurrencyError),
    #[error("{0}")]
    SubscriberError(#[from] SubscriberError),
    #[error("{0}")]
    ExchangePriceCacheError(#[from] ExchangePriceCacheError),
}
