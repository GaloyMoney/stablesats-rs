use thiserror::Error;

use shared::time::*;

use crate::cache::OrderBookCacheError;
#[derive(Error, Debug)]
pub enum ExchangePriceCacheError {
    #[error("StalePrice: last update was at {0}")]
    StalePrice(TimeStamp),
    #[error("No price data available")]
    NoPriceAvailable,
    #[error("OrderBook: {0:?}")]
    OrderBookCache(#[from] OrderBookCacheError),
}
