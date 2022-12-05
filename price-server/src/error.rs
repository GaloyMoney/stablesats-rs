use crate::exchange_order_book_cache;
use crate::price_mixer;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExchangePriceCacheError {
    #[error("PriceTick: {0}")]
    PriceTickCache(#[from] price_mixer::PriceTickCacheError),
    #[error("OrderBook: {0:?}")]
    OrderBookCache(#[from] exchange_order_book_cache::OrderBookCacheError),
}
