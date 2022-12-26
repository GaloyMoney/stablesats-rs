pub mod config;

use chrono::Duration;
use opentelemetry::trace::{SpanContext, TraceContextExt};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;

use shared::{
    payload::*,
    pubsub::{self, CorrelationId},
    time::*,
};

use crate::currency::*;
pub use config::*;

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
    config: ExchangePriceCacheConfig,
}

impl ExchangePriceCache {
    pub fn new(config: ExchangePriceCacheConfig) -> Self {
        Self {
            inner: Arc::new(RwLock::new(ExchangePriceCacheInner::new(
                config.stale_after,
            ))),
            config,
        }
    }

    pub async fn apply_update(&self, message: pubsub::Envelope<OkexBtcUsdSwapPricePayload>) {
        self.inner.write().await.update_price(message);
    }

    pub async fn latest_tick(&self) -> Result<BtcSatTick, ExchangePriceCacheError> {
        if let Some(mock_price) = self.config.dev_mock_price_btc_in_usd {
            let price = PriceRatioRaw::from_one_btc_in_usd_price(mock_price);
            let cent_price = UsdCents::try_from(price).expect("couldn't create mack UsdCents");
            return Ok(BtcSatTick {
                timestamp: TimeStamp::now(),
                correlation_id: CorrelationId::new(),
                span_context: Span::current().context().span().span_context().clone(),
                ask_price_of_one_sat: cent_price.clone(),
                bid_price_of_one_sat: cent_price,
            });
        }
        let tick = self.inner.read().await.latest_tick()?;
        let span = Span::current();
        span.add_link(tick.span_context.clone());
        span.record(
            "correlation_id",
            &tracing::field::display(tick.correlation_id),
        );
        Ok(tick)
    }
}

#[derive(Clone)]
pub struct BtcSatTick {
    timestamp: TimeStamp,
    correlation_id: CorrelationId,
    span_context: SpanContext,
    ask_price_of_one_sat: UsdCents,
    bid_price_of_one_sat: UsdCents,
}

impl BtcSatTick {
    pub fn mid_price_of_one_sat(&self) -> UsdCents {
        (&self.bid_price_of_one_sat + &self.ask_price_of_one_sat) / 2
    }

    pub fn sell_usd(&self) -> CurrencyConverter {
        CurrencyConverter::new(&self.ask_price_of_one_sat)
    }

    pub fn buy_usd(&self) -> CurrencyConverter {
        CurrencyConverter::new(&self.bid_price_of_one_sat)
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

    fn update_price(&mut self, message: pubsub::Envelope<OkexBtcUsdSwapPricePayload>) {
        let payload = message.payload.0;
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
                correlation_id: message.meta.correlation_id,
                span_context: Span::current().context().span().span_context().clone(),
                ask_price_of_one_sat,
                bid_price_of_one_sat,
            });
        }
    }

    fn latest_tick(&self) -> Result<BtcSatTick, ExchangePriceCacheError> {
        if let Some(ref tick) = self.tick {
            if tick.timestamp.duration_since() > self.stale_after {
                return Err(ExchangePriceCacheError::StalePrice(tick.timestamp));
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
    fn test_mid_price_of_one_sat() {
        let tick = BtcSatTick {
            timestamp: TimeStamp::now(),
            correlation_id: CorrelationId::new(),
            span_context: SpanContext::empty_context(),
            bid_price_of_one_sat: UsdCents::from_major(5000),
            ask_price_of_one_sat: UsdCents::from_major(10000),
        };

        assert_eq!(tick.mid_price_of_one_sat(), UsdCents::from_major(7500));
    }
}
