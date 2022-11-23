use chrono::Duration;
use opentelemetry::trace::{SpanContext, TraceContextExt};
use rust_decimal::Decimal;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;

use crate::currency::*;
use rust_decimal_macros::dec;
use shared::{payload::*, pubsub::CorrelationId, time::*};

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

    pub async fn apply_update(&self, payload: PriceMessagePayload, id: CorrelationId) {
        self.inner.write().await.update_price(payload, id);
    }

    pub async fn latest_tick(&self) -> Result<BtcSatTick, ExchangePriceCacheError> {
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

#[derive(Clone, Debug)]
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

    pub fn apply_weight(tick: BtcSatTick, weight: Decimal) -> Self {
        BtcSatTick {
            ask_price_of_one_sat: tick.ask_price_of_one_sat * weight,
            bid_price_of_one_sat: tick.bid_price_of_one_sat * weight,
            ..tick
        }
    }

    pub fn merge(ticks: Vec<BtcSatTick>) -> Option<Self> {
        if ticks.is_empty() {
            return None;
        }

        let mut ask_price = dec!(0);
        let mut bid_price = dec!(0);

        for item in ticks.iter() {
            ask_price += item.ask_price_of_one_sat.amount();
            bid_price += item.bid_price_of_one_sat.amount();
        }
        let amount_items = Decimal::from(ticks.len());

        ask_price /= amount_items;
        bid_price /= amount_items;

        let first = ticks.get(0)?;
        Some(BtcSatTick {
            ask_price_of_one_sat: UsdCents::from_decimal(ask_price),
            bid_price_of_one_sat: UsdCents::from_decimal(bid_price),
            ..first.clone()
        })
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

    fn update_price(&mut self, payload: PriceMessagePayload, id: CorrelationId) {
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
                correlation_id: id,
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
