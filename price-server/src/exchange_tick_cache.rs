use chrono::Duration;
use opentelemetry::trace::{SpanContext, TraceContextExt};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;

use crate::{currency::*, price_mixer::*};
use shared::{payload::*, pubsub::CorrelationId, time::*};

#[derive(Clone)]
pub struct ExchangeTickCache {
    inner: Arc<RwLock<ExchangePriceCacheInner>>,
    config: ExchangePriceCacheConfig,
}

impl ExchangeTickCache {
    pub fn new(config: ExchangePriceCacheConfig) -> Self {
        Self {
            inner: Arc::new(RwLock::new(ExchangePriceCacheInner::new(
                config.stale_after,
            ))),
            config,
        }
    }

    pub async fn apply_update(&self, payload: PriceMessagePayload, id: CorrelationId) {
        self.inner.write().await.update_price(payload, id);
    }
}

#[async_trait::async_trait]
impl PriceProvider for ExchangeTickCache {
    async fn latest(&self) -> Result<Box<dyn SidePicker>, ExchangePriceCacheError> {
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
        let inner = self.inner.read().await;
        let tick = inner.latest_tick()?;

        let span = Span::current();
        span.add_link(tick.span_context.clone());
        span.record(
            "correlation_id",
            &tracing::field::display(tick.correlation_id),
        );
        Ok(Box::new(tick))
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

impl SidePicker for BtcSatTick {
    fn buy_usd<'a>(&'a self) -> Box<dyn VolumePicker + 'a> {
        Box::new(CurrencyConverter::new(&self.bid_price_of_one_sat))
    }

    fn sell_usd<'a>(&'a self) -> Box<dyn VolumePicker + 'a> {
        Box::new(CurrencyConverter::new(&self.ask_price_of_one_sat))
    }

    fn mid_price_of_one_sat(&self) -> UsdCents {
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
        let _tick = BtcSatTick {
            timestamp: TimeStamp::now(),
            correlation_id: CorrelationId::new(),
            span_context: SpanContext::empty_context(),
            bid_price_of_one_sat: UsdCents::from_major(5000),
            ask_price_of_one_sat: UsdCents::from_major(10000),
        };

        assert_eq!(UsdCents::from_major(7500), _tick.mid_price_of_one_sat());
    }
}
