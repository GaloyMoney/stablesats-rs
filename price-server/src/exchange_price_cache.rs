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

    pub async fn price_of_one_sat(&self) -> Result<UsdCents, ExchangePriceCacheError> {
        self.inner.read().await.price_of_one_sat()
    }
}

struct BtcSatTick {
    timestamp: TimeStamp,
    price_of_one_sat: UsdCents,
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
        if let Ok(price_of_one_sat) = UsdCents::try_from(payload.ask_price) {
            self.tick = Some(BtcSatTick {
                timestamp: payload.timestamp,
                price_of_one_sat,
            });
        }
    }

    fn price_of_one_sat(&self) -> Result<UsdCents, ExchangePriceCacheError> {
        if let Some(ref tick) = self.tick {
            if &TimeStamp::now() - &tick.timestamp > self.stale_after {
                return Err(ExchangePriceCacheError::StalePrice(tick.timestamp.clone()));
            }
            return Ok(tick.price_of_one_sat.clone());
        }
        Err(ExchangePriceCacheError::NoPriceAvailable)
    }
}
