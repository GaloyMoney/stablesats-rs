use chrono::Duration;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

use shared::{currency::*, payload::*, time::*};

#[derive(Error, Debug)]
pub enum ExchangePriceCacheError {
    #[error("StalePrice: last update was at {0}")]
    StalePrice(TimeStamp),
    #[error("No price data available")]
    NoPriceAvailable,
}

#[derive(Clone)]
pub struct ExchangePriceCache {
    inner: Arc<RwLock<ExchangePriceCacheInner>>,
}

impl ExchangePriceCache {
    pub fn new(stale_after: Duration) -> Self {
        Self {
            inner: Arc::new(RwLock::new(ExchangePriceCacheInner::new(stale_after))),
        }
    }

    pub async fn apply_update(&self, payload: OkexBtcUsdSwapPricePayload) {
        self.inner.write().await.update_price(payload);
    }

    pub async fn ask_price_of_one_sat(&self) -> Result<UsdCents, ExchangePriceCacheError> {
        self.inner.read().await.ask_price_of_one_sat()
    }

    pub async fn bid_price_of_one_sat(&self) -> Result<UsdCents, ExchangePriceCacheError> {
        self.inner.read().await.bid_price_of_one_sat()
    }
}

struct BtcSatTick {
    timestamp: TimeStamp,
    ask_price_of_one_sat: UsdCents,
    bid_price_of_one_sat: UsdCents,
}

struct ExchangePriceCacheInner {
    stale_after: Duration,
    tick: Option<BtcSatTick>,
}

impl ExchangePriceCacheInner {
    fn new(stale_after: Duration) -> Self {
        Self {
            stale_after,
            tick: None,
        }
    }

    fn update_price(&mut self, payload: impl Into<PriceMessagePayload>) {
        let payload = payload.into();
        if let Some(ref tick) = self.tick {
            if tick.timestamp > payload.timestamp {
                return;
            }
        }
        if let (Ok(ask_price_of_one_sat), Ok(bid_price_of_one_sat)) = (
            UsdCents::try_from(payload.ask_price),
            UsdCents::try_from(payload.bid_price),
        ) {
            self.tick = Some(BtcSatTick {
                timestamp: payload.timestamp,
                ask_price_of_one_sat,
                bid_price_of_one_sat,
            });
        }
    }

    fn ask_price_of_one_sat(&self) -> Result<UsdCents, ExchangePriceCacheError> {
        if let Some(ref tick) = self.tick {
            if &TimeStamp::now() - &tick.timestamp > self.stale_after {
                return Err(ExchangePriceCacheError::StalePrice(tick.timestamp.clone()));
            }
            return Ok(tick.ask_price_of_one_sat.clone());
        }
        Err(ExchangePriceCacheError::NoPriceAvailable)
    }

    fn bid_price_of_one_sat(&self) -> Result<UsdCents, ExchangePriceCacheError> {
        if let Some(ref tick) = self.tick {
            if &TimeStamp::now() - &tick.timestamp > self.stale_after {
                return Err(ExchangePriceCacheError::StalePrice(tick.timestamp.clone()));
            }
            return Ok(tick.bid_price_of_one_sat.clone());
        }
        Err(ExchangePriceCacheError::NoPriceAvailable)
    }
}
