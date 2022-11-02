mod error;

use chrono::Duration;
use futures::stream::StreamExt;
use tracing::{info_span, instrument, Instrument};

use shared::{
    health::HealthCheckTrigger,
    payload::{OkexBtcUsdSwapOrderBookPayload, OkexBtcUsdSwapPricePayload},
    pubsub::*,
};

use super::exchange_price_cache::ExchangePriceCache;

use crate::SnapshotCache;
pub use crate::{currency::*, fee_calculator::*};
pub use error::*;

pub struct PriceApp {
    price_cache: SnapshotCache,
    fee_calculator: FeeCalculator,
}

impl PriceApp {
    pub async fn run(
        mut health_check_trigger: HealthCheckTrigger,
        fee_calc_cfg: FeeCalculatorConfig,
        pubsub_cfg: PubSubConfig,
    ) -> Result<Self, PriceAppError> {
        let subscriber = Subscriber::new(pubsub_cfg).await?;
        let mut stream = subscriber
            .subscribe::<OkexBtcUsdSwapOrderBookPayload>()
            .await?;
        tokio::spawn(async move {
            while let Some(check) = health_check_trigger.next().await {
                check
                    .send(subscriber.healthy(Duration::seconds(20)).await)
                    .expect("Couldn't send response");
            }
        });

        let price_cache = SnapshotCache::new(Duration::seconds(30));
        let fee_calculator = FeeCalculator::new(fee_calc_cfg);
        let app = Self {
            price_cache: price_cache.clone(),
            fee_calculator,
        };

        let _ = tokio::spawn(async move {
            while let Some(msg) = stream.next().await {
                let span = info_span!(
                    "price_tick_received",
                    message_type = %msg.payload_type,
                    correlation_id = %msg.meta.correlation_id
                );
                shared::tracing::inject_tracing_data(&span, &msg.meta.tracing_data);

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
            .latest_snapshot()
            .await?
            .buy_usd()
            .cents_from_sats(sats);
        Ok(self.fee_calculator.decrease_by_immediate_fee(cents).floor())
    }

    #[instrument(skip_all, fields(correlation_id, amount = %sats.amount()), ret, err)]
    pub async fn get_cents_from_sats_for_immediate_sell(
        &self,
        sats: Sats,
    ) -> Result<UsdCents, PriceAppError> {
        let cents = self
            .price_cache
            .latest_snapshot()
            .await?
            .sell_usd()
            .cents_from_sats(sats);
        Ok(self.fee_calculator.increase_by_immediate_fee(cents).ceil())
    }

    #[instrument(skip_all, fields(correlation_id, amount = %sats.amount()), ret, err)]
    pub async fn get_cents_from_sats_for_future_buy(
        &self,
        sats: Sats,
    ) -> Result<UsdCents, PriceAppError> {
        let cents = self
            .price_cache
            .latest_snapshot()
            .await?
            .buy_usd()
            .cents_from_sats(sats);
        Ok(self.fee_calculator.decrease_by_delayed_fee(cents).floor())
    }

    #[instrument(skip_all, fields(correlation_id, amount = %sats.amount()), ret, err)]
    pub async fn get_cents_from_sats_for_future_sell(
        &self,
        sats: Sats,
    ) -> Result<UsdCents, PriceAppError> {
        let cents = self
            .price_cache
            .latest_snapshot()
            .await?
            .sell_usd()
            .cents_from_sats(sats);
        Ok(self.fee_calculator.increase_by_delayed_fee(cents).ceil())
    }

    #[instrument(skip_all, fields(correlation_id, amount = %cents.amount()), ret, err)]
    pub async fn get_sats_from_cents_for_immediate_buy(
        &self,
        cents: UsdCents,
    ) -> Result<Sats, PriceAppError> {
        let sats = self
            .price_cache
            .latest_snapshot()
            .await?
            .buy_usd()
            .sats_from_cents(cents);
        Ok(self.fee_calculator.increase_by_immediate_fee(sats).ceil())
    }

    #[instrument(skip_all, fields(correlation_id, amount = %cents.amount()), ret, err)]
    pub async fn get_sats_from_cents_for_immediate_sell(
        &self,
        cents: UsdCents,
    ) -> Result<Sats, PriceAppError> {
        let sats = self
            .price_cache
            .latest_snapshot()
            .await?
            .sell_usd()
            .sats_from_cents(cents);
        Ok(self.fee_calculator.decrease_by_immediate_fee(sats).floor())
    }

    #[instrument(skip_all, fields(correlation_id, amount = %cents.amount()), ret, err)]
    pub async fn get_sats_from_cents_for_future_buy(
        &self,
        cents: UsdCents,
    ) -> Result<Sats, PriceAppError> {
        let sats = self
            .price_cache
            .latest_snapshot()
            .await?
            .buy_usd()
            .sats_from_cents(cents);
        Ok(self.fee_calculator.increase_by_delayed_fee(sats).ceil())
    }

    #[instrument(skip_all, fields(correlation_id, amount = %cents.amount()), ret, err)]
    pub async fn get_sats_from_cents_for_future_sell(
        &self,
        cents: UsdCents,
    ) -> Result<Sats, PriceAppError> {
        let sats = self
            .price_cache
            .latest_snapshot()
            .await?
            .sell_usd()
            .sats_from_cents(cents);
        Ok(self.fee_calculator.decrease_by_delayed_fee(sats).floor())
    }

    #[instrument(skip_all, fields(correlation_id), ret, err)]
    pub async fn get_cents_per_sat_exchange_mid_rate(&self) -> Result<f64, PriceAppError> {
        let cents_per_sat = self
            .price_cache
            .latest_snapshot()
            .await?
            .mid_price_of_one_sat();
        Ok(f64::try_from(cents_per_sat)?)
    }
}
