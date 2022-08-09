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

    pub async fn latest_tick(&self) -> Result<BtcSatTick, ExchangePriceCacheError> {
        self.inner.read().await.latest_tick()
    }
}

#[derive(Clone)]
pub struct BtcSatTick {
    timestamp: TimeStamp,
    pub ask_price_of_one_sat: UsdCents,
    pub bid_price_of_one_sat: UsdCents,
}

impl BtcSatTick {
    pub fn mid_price_of_one_sat(&self) -> UsdCents {
        (&self.bid_price_of_one_sat + &self.ask_price_of_one_sat) / 2
    }
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

    fn latest_tick(&self) -> Result<BtcSatTick, ExchangePriceCacheError> {
        if let Some(ref tick) = self.tick {
            if tick.timestamp.duration_since() > self.stale_after {
                return Err(ExchangePriceCacheError::StalePrice(tick.timestamp.clone()));
            }
            return Ok(tick.clone());
        }
        Err(ExchangePriceCacheError::NoPriceAvailable)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn test_price_of_one_sat() {
        let tick = BtcSatTick {
            timestamp: TimeStamp::now(),
            bid_price_of_one_sat: UsdCents::from_major(5000),
            ask_price_of_one_sat: UsdCents::from_major(10000),
        };

        assert_eq!(tick.mid_price_of_one_sat(), UsdCents::from_major(7500));
    }
}
