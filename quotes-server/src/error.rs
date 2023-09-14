use shared::time::*;
use thiserror::Error;

#[allow(clippy::large_enum_variant)]
#[derive(Error, Debug)]
pub enum QuotesAppError {}

#[derive(Error, Debug)]
pub enum ExchangePriceCacheError {
    #[error("StalePrice: last update was at {0}")]
    StalePrice(TimeStamp),
    #[error("No price data available")]
    NoPriceAvailable,
}
