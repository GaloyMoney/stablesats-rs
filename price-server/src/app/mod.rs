mod error;

use chrono::Duration;
use futures::stream::StreamExt;
use opentelemetry::{propagation::TextMapPropagator, sdk::propagation::TraceContextPropagator};
use tracing::{info_span, instrument, Instrument};
use tracing_opentelemetry::OpenTelemetrySpanExt;

use shared::{currency::*, payload::OkexBtcUsdSwapPricePayload, pubsub::*};

use super::exchange_price_cache::ExchangePriceCache;

pub use crate::fee_calculator::*;
pub use error::*;

pub struct PriceApp {
    price_cache: ExchangePriceCache,
    fee_calculator: FeeCalculator,
}

impl PriceApp {
    pub async fn run(
        fee_calc_cfg: FeeCalculatorConfig,
        pubsub_cfg: PubSubConfig,
    ) -> Result<Self, PriceAppError> {
        let subscriber = Subscriber::new(pubsub_cfg).await?;
        let mut stream = subscriber.subscribe::<OkexBtcUsdSwapPricePayload>().await?;

        let price_cache = ExchangePriceCache::new(Duration::seconds(30));
        let fee_calculator = FeeCalculator::new(fee_calc_cfg);
        let app = Self {
            price_cache: price_cache.clone(),
            fee_calculator,
        };

        let _ = tokio::spawn(async move {
            let propagator = TraceContextPropagator::new();

            while let Some(msg) = stream.next().await {
                let span = info_span!(
                    "price_tick_received",
                    message_type = %msg.payload_type,
                    correlation_id = %msg.meta.correlation_id
                );
                let context = propagator.extract(&msg.meta.tracing_data);
                span.set_parent(context);

                async {
                    price_cache.apply_update(msg).await;
                }
                .instrument(span)
                .await;
            }
        });
        Ok(app)
    }

    #[instrument(skip_all, fields(correlation_id, amount = %sats.amount()), ret, err)]
    pub async fn get_cents_from_sats_for_immediate_buy(
        &self,
        sats: Sats,
    ) -> Result<UsdCents, PriceAppError> {
        let cents = self
            .price_cache
            .latest_tick()
            .await?
            .ask_price()
            .cents_from_sats(sats);
        Ok(self.fee_calculator.apply_immediate_fee(cents))
    }

    #[instrument(skip_all, fields(correlation_id, amount = %sats.amount()), ret, err)]
    pub async fn get_cents_from_sats_for_immediate_sell(
        &self,
        sats: Sats,
    ) -> Result<UsdCents, PriceAppError> {
        let cents = self
            .price_cache
            .latest_tick()
            .await?
            .bid_price()
            .cents_from_sats(sats);
        Ok(self.fee_calculator.apply_immediate_fee(cents))
    }

    #[instrument(skip_all, fields(correlation_id, amount = %sats.amount()), ret, err)]
    pub async fn get_cents_from_sats_for_future_buy(
        &self,
        sats: Sats,
    ) -> Result<UsdCents, PriceAppError> {
        let cents = self
            .price_cache
            .latest_tick()
            .await?
            .ask_price()
            .cents_from_sats(sats);
        Ok(self.fee_calculator.apply_delayed_fee(cents))
    }

    #[instrument(skip_all, fields(correlation_id, amount = %sats.amount()), ret, err)]
    pub async fn get_cents_from_sats_for_future_sell(
        &self,
        sats: Sats,
    ) -> Result<UsdCents, PriceAppError> {
        let cents = self
            .price_cache
            .latest_tick()
            .await?
            .bid_price()
            .cents_from_sats(sats);
        Ok(self.fee_calculator.apply_delayed_fee(cents))
    }

    #[instrument(skip_all, fields(correlation_id, amount = %cents.amount()), ret, err)]
    pub async fn get_sats_from_cents_for_immediate_buy(
        &self,
        cents: UsdCents,
    ) -> Result<Sats, PriceAppError> {
        let sats = self
            .price_cache
            .latest_tick()
            .await?
            .ask_price()
            .sats_from_cents(cents);
        Ok(self.fee_calculator.apply_immediate_fee(sats))
    }

    #[instrument(skip_all, fields(correlation_id, amount = %cents.amount()), ret, err)]
    pub async fn get_sats_from_cents_for_immediate_sell(
        &self,
        cents: UsdCents,
    ) -> Result<Sats, PriceAppError> {
        let sats = self
            .price_cache
            .latest_tick()
            .await?
            .bid_price()
            .sats_from_cents(cents);
        Ok(self.fee_calculator.apply_immediate_fee(sats))
    }

    #[instrument(skip_all, fields(correlation_id, amount = %cents.amount()), ret, err)]
    pub async fn get_sats_from_cents_for_future_buy(
        &self,
        cents: UsdCents,
    ) -> Result<Sats, PriceAppError> {
        let sats = self
            .price_cache
            .latest_tick()
            .await?
            .ask_price()
            .sats_from_cents(cents);
        Ok(self.fee_calculator.apply_delayed_fee(sats))
    }

    #[instrument(skip_all, fields(correlation_id, amount = %cents.amount()), ret, err)]
    pub async fn get_sats_from_cents_for_future_sell(
        &self,
        cents: UsdCents,
    ) -> Result<Sats, PriceAppError> {
        let sats = self
            .price_cache
            .latest_tick()
            .await?
            .bid_price()
            .sats_from_cents(cents);
        Ok(self.fee_calculator.apply_delayed_fee(sats))
    }

    #[instrument(skip_all, fields(correlation_id, ret, err))]
    pub async fn get_cents_per_sat_exchange_mid_rate(&self) -> Result<f64, PriceAppError> {
        let cents_per_sat = self.price_cache.latest_tick().await?.mid_price_of_one_sat();
        Ok(f64::try_from(cents_per_sat)?)
    }
}
