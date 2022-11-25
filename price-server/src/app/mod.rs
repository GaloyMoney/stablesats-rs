mod error;

use std::{collections::HashMap, sync::Arc};

use chrono::Duration;
use futures::stream::StreamExt;
use rust_decimal::Decimal;
use tokio::sync::RwLock;
use tracing::{info_span, instrument, Instrument};

use shared::{
    exchanges_config::{ExchangeConfigAll, ExchangeType},
    health::HealthCheckTrigger,
    payload::{
        KolliderBtcUsdSwapPricePayload, OkexBtcUsdSwapPricePayload, KOLLIDER_EXCHANGE_ID,
        OKEX_EXCHANGE_ID,
    },
    pubsub::*,
};

use super::exchange_price_cache::ExchangePriceCache;

pub use crate::{currency::*, fee_calculator::*};
use crate::{exchange_price_cache::BtcSatTick, ExchangePriceCacheError};
pub use error::*;

pub struct PriceApp {
    all_price_caches: PricesCache,
    fee_calculator: FeeCalculator,
}

struct PricesCache {
    caches: HashMap<String, Box<ExchangePriceCache>>,
    exchange_configs: ExchangeConfigAll,
}

impl PricesCache {
    pub async fn latest_tick(&self) -> Result<BtcSatTick, ExchangePriceCacheError> {
        let mut weighted_ticks = vec![];

        let okex_cfg = self.exchange_configs.okex.as_ref().unwrap();
        let tick = self
            .caches
            .get(&OKEX_EXCHANGE_ID.to_string())
            .unwrap()
            .latest_tick()
            .await?;
        weighted_ticks.push(BtcSatTick::apply_weight(tick, okex_cfg.weight));

        BtcSatTick::merge(weighted_ticks).ok_or(ExchangePriceCacheError::NoPriceAvailable)
    }
}

impl PriceApp {
    pub async fn run(
        health_check_trigger: HealthCheckTrigger,
        fee_calc_cfg: FeeCalculatorConfig,
        pubsub_cfg: PubSubConfig,
        exchanges_cfg: ExchangeConfigAll,
    ) -> Result<Self, PriceAppError> {
        let ht = Arc::new(RwLock::new(health_check_trigger));
        let mut prices_cache: PricesCache = PricesCache {
            caches: HashMap::new(),
            exchange_configs: exchanges_cfg,
        };

        if prices_cache.exchange_configs.kollider.is_some() {
            let kollider_price_cache = ExchangePriceCache::new(Duration::seconds(30));
            prices_cache.caches.insert(
                KOLLIDER_EXCHANGE_ID.to_string(),
                Box::new(kollider_price_cache.clone()),
            );
            Self::subscribe_kollider(pubsub_cfg.clone(), ht.clone(), kollider_price_cache.clone())
                .await?
        }

        if prices_cache.exchange_configs.okex.is_some() {
            let okex_price_cache = ExchangePriceCache::new(Duration::seconds(30));
            prices_cache.caches.insert(
                OKEX_EXCHANGE_ID.to_string(),
                Box::new(okex_price_cache.clone()),
            );
            Self::subscribe_okex(pubsub_cfg.clone(), ht.clone(), okex_price_cache.clone()).await?
        }

        let fee_calculator = FeeCalculator::new(fee_calc_cfg);
        let app = Self {
            all_price_caches: prices_cache,
            fee_calculator,
        };

        Ok(app)
    }

    async fn subscribe_okex(
        pubsub_cfg: PubSubConfig,
        health_check_trigger: Arc<RwLock<HealthCheckTrigger>>,
        price_cache: ExchangePriceCache,
    ) -> Result<(), PriceAppError> {
        let subscriber = Subscriber::new(pubsub_cfg).await?;
        let mut stream = subscriber
            .subscribe::<OkexBtcUsdSwapPricePayload>()
            .await
            .unwrap();
        tokio::spawn(async move {
            while let Some(check) = health_check_trigger.write().await.next().await {
                check
                    .send(subscriber.healthy(Duration::seconds(20)).await)
                    .expect("Couldn't send response");
            }
        });

        let _ = tokio::spawn(async move {
            while let Some(msg) = stream.next().await {
                let span = info_span!(
                    "price_tick_received",
                    message_type = %msg.payload_type,
                    correlation_id = %msg.meta.correlation_id
                );
                shared::tracing::inject_tracing_data(&span, &msg.meta.tracing_data);

                async {
                    price_cache
                        .apply_update(msg.payload.0, msg.meta.correlation_id)
                        .await;
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
        price_cache: ExchangePriceCache,
    ) -> Result<(), PriceAppError> {
        let subscriber = Subscriber::new(pubsub_cfg).await?;
        let mut stream = subscriber
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

        let _ = tokio::spawn(async move {
            while let Some(msg) = stream.next().await {
                let span = info_span!(
                    "price_tick_received",
                    message_type = %msg.payload_type,
                    correlation_id = %msg.meta.correlation_id
                );
                shared::tracing::inject_tracing_data(&span, &msg.meta.tracing_data);

                async {
                    price_cache
                        .apply_update(msg.payload.0, msg.meta.correlation_id)
                        .await;
                }
                .instrument(span)
                .await;
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
            .all_price_caches
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
            .all_price_caches
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
            .all_price_caches
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
            .all_price_caches
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
            .all_price_caches
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
            .all_price_caches
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
            .all_price_caches
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
            .all_price_caches
            .latest_tick()
            .await?
            .sell_usd()
            .sats_from_cents(cents);
        Ok(self.fee_calculator.decrease_by_delayed_fee(sats).floor())
    }

    #[instrument(skip_all, fields(correlation_id), ret, err)]
    pub async fn get_cents_per_sat_exchange_mid_rate(&self) -> Result<f64, PriceAppError> {
        let cents_per_sat = self
            .all_price_caches
            .latest_tick()
            .await?
            .mid_price_of_one_sat();
        Ok(f64::try_from(cents_per_sat)?)
    }
}
