mod error;

use std::sync::Arc;

use chrono::Duration;
use futures::stream::StreamExt;
use tokio::sync::RwLock;
use tracing::{info_span, instrument, Instrument};

use shared::{
    exchanges_config::{ExchangeType, ExchangesConfig},
    health::HealthCheckTrigger,
    payload::{
        KolliderBtcUsdSwapPricePayload, OkexBtcUsdSwapOrderBookPayload, OkexBtcUsdSwapPricePayload,
    },
    pubsub::*,
};

use super::exchange_price_cache::ExchangePriceCache;

pub use crate::{currency::*, fee_calculator::*};
use crate::{exchange_price_cache::ExchangePriceCache, OrderBookCache};
pub use error::*;

pub struct PriceApp {
    price_cache: ExchangePriceCache,
    fee_calculator: FeeCalculator,
}

impl PriceApp {
    pub async fn run(
        health_check_trigger: HealthCheckTrigger,
        fee_calc_cfg: FeeCalculatorConfig,
        pubsub_cfg: PubSubConfig,
        exchanges_cfg: ExchangesConfig,
    ) -> Result<Self, PriceAppError> {
        let ht = Arc::new(RwLock::new(health_check_trigger));

        let order_book_cache = OrderBookCache::new(Duration::seconds(30));

        for exchange in exchanges_cfg {
            match exchange.config {
                ExchangeType::Kollider(_) => {
                    Self::subscribe_kollider(pubsub_cfg.clone(), ht.clone()).await?
                }
                ExchangeType::OkEx(_) => {
                    Self::subscribe_okex(pubsub_cfg.clone(), ht.clone(), order_book_cache.clone())
                        .await?
                }
            }
        }

        let kollider_price_cache = ExchangePriceCache::new(Duration::seconds(30));

        let fee_calculator = FeeCalculator::new(fee_calc_cfg);
        let app = Self {
            price_cache: order_book_cache.clone(),
            fee_calculator,
        };

        Ok(app)
    }

    async fn subscribe_okex(
        pubsub_cfg: PubSubConfig,
        health_check_trigger: Arc<RwLock<HealthCheckTrigger>>,
        order_book_cache: OrderBookCache,
    ) -> Result<(), PriceAppError> {
        let subscriber = Subscriber::new(pubsub_cfg).await?;
        let mut stream = subscriber
            .subscribe::<OkexBtcUsdSwapOrderBookPayload>()
            .await?;
        tokio::spawn(async move {
            while let Some(check) = health_check_trigger.write().await.next().await {
                check
                    .send(subscriber.healthy(Duration::seconds(20)).await)
                    .expect("Couldn't send response");
            }
        });

        let order_book_cache = OrderBookCache::new(Duration::seconds(30));
        let fee_calculator = FeeCalculator::new(fee_calc_cfg);
        let app = Self {
            price_cache: order_book_cache.clone(),
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

        Ok(())
    }

    async fn subscribe_kollider(
        pubsub_cfg: PubSubConfig,
        health_check_trigger: Arc<RwLock<HealthCheckTrigger>>,
    ) -> Result<(), PriceAppError> {
        let subscriber = Subscriber::new(pubsub_cfg).await?;
        let _stream = subscriber
            .subscribe::<KolliderBtcUsdSwapPricePayload>()
            .await
            .unwrap();
        tokio::spawn(async move {
            while let Some(check) = health_check_trigger.write().await.next().await {
                check
                    .send(subscriber.healthy(Duration::seconds(20)).await)
                    .expect("Couldn't send response");
            }
        });
        Ok(())
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
            .latest_tick()
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
            .latest_tick()
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
            .latest_tick()
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
            .latest_tick()
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
            .latest_tick()
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
            .latest_tick()
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
            .latest_tick()
            .await?
            .sell_usd()
            .sats_from_cents(cents);
        Ok(self.fee_calculator.decrease_by_delayed_fee(sats).floor())
    }

    #[instrument(skip_all, fields(correlation_id), ret, err)]
    pub async fn get_cents_per_sat_exchange_mid_rate(&self) -> Result<f64, PriceAppError> {
        let cents_per_sat = self.price_cache.latest_tick().await?.mid_price_of_one_sat();
        Ok(f64::try_from(cents_per_sat)?)
    }
}
