mod error;

use std::{collections::HashMap, sync::Arc};

use chrono::Duration;
use futures::stream::StreamExt;
use rust_decimal::Decimal;
use tokio::sync::RwLock;
use tracing::{info_span, instrument, Instrument};

use shared::{
    exchanges_config::ExchangeConfigAll,
    health::HealthCheckTrigger,
    payload::{
        KolliderBtcUsdSwapPricePayload, OkexBtcUsdSwapOrderBookPayload, OkexBtcUsdSwapPricePayload,
        KOLLIDER_EXCHANGE_ID, OKEX_EXCHANGE_ID,
    },
    pubsub::*,
};

use crate::OrderBookCache;
pub use crate::{currency::*, fee_calculator::*};
use crate::{
    exchange_tick_cache::ExchangeTickCache,
    price_mixer::{PriceMixer, PriceProvider},
};
pub use error::*;

pub struct PriceApp {
    price_mixer: PriceMixer,
    order_book_cache: OrderBookCache,
    fee_calculator: FeeCalculator,
}

impl PriceApp {
    pub async fn run(
        health_check_trigger: HealthCheckTrigger,
        fee_calc_cfg: FeeCalculatorConfig,
        pubsub_cfg: PubSubConfig,
        exchanges_cfg: ExchangeConfigAll,
    ) -> Result<Self, PriceAppError> {
        let health_check_trigger = Arc::new(RwLock::new(health_check_trigger));
        let mut providers: HashMap<String, (Box<dyn PriceProvider + Sync + Send>, Decimal)> =
            HashMap::new();
        let order_book_cache = OrderBookCache::new(Duration::seconds(30));

        if let Some(config) = exchanges_cfg.kollider.as_ref() {
            let kollider_price_cache = ExchangeTickCache::new(Duration::seconds(30));
            Self::subscribe_kollider(
                pubsub_cfg.clone(),
                health_check_trigger.clone(),
                kollider_price_cache.clone(),
            )
            .await?;
            providers.insert(
                KOLLIDER_EXCHANGE_ID.to_string(),
                (Box::new(kollider_price_cache), config.weight),
            );
        }

        if let Some(config) = exchanges_cfg.okex.as_ref() {
            let okex_price_cache = ExchangeTickCache::new(Duration::seconds(30));
            Self::subscribe_okex(
                pubsub_cfg.clone(),
                health_check_trigger.clone(),
                okex_price_cache.clone(),
            )
            .await?;
            providers.insert(
                OKEX_EXCHANGE_ID.to_string(),
                (Box::new(okex_price_cache), config.weight),
            );
        }

        let _ = Self::subscribe_okex_order_book(
            pubsub_cfg,
            health_check_trigger,
            order_book_cache.clone(),
        )
        .await;

        let fee_calculator = FeeCalculator::new(fee_calc_cfg);
        let app = Self {
            price_mixer: PriceMixer::new(providers),
            order_book_cache,
            fee_calculator,
        };

        Ok(app)
    }

    async fn subscribe_okex(
        pubsub_cfg: PubSubConfig,
        health_check_trigger: Arc<RwLock<HealthCheckTrigger>>,
        price_cache: ExchangeTickCache,
    ) -> Result<(), PriceAppError> {
        let subscriber = Subscriber::new(pubsub_cfg).await?;
        let mut stream = subscriber.subscribe::<OkexBtcUsdSwapPricePayload>().await?;
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

    async fn subscribe_okex_order_book(
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

        let _ = tokio::spawn(async move {
            while let Some(msg) = stream.next().await {
                let span = info_span!(
                    "order_book_received",
                    message_type = %msg.payload_type,
                    correlation_id = %msg.meta.correlation_id
                );
                shared::tracing::inject_tracing_data(&span, &msg.meta.tracing_data);

                async {
                    order_book_cache.apply_update(msg).await;
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
        price_cache: ExchangeTickCache,
    ) -> Result<(), PriceAppError> {
        let subscriber = Subscriber::new(pubsub_cfg).await?;
        let mut stream = subscriber
            .subscribe::<KolliderBtcUsdSwapPricePayload>()
            .await?;
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
        let cents = UsdCents::from_decimal(
            self.price_mixer
                .apply(|p| *p.buy_usd().cents_from_sats(sats.clone()).amount())
                .await?,
        );

        Ok(self.fee_calculator.decrease_by_immediate_fee(cents).floor())
    }

    #[instrument(skip_all, fields(correlation_id, amount = %sats.amount()), ret, err)]
    pub async fn get_cents_from_sats_for_immediate_sell(
        &self,
        sats: Sats,
    ) -> Result<UsdCents, PriceAppError> {
        let cents = UsdCents::from_decimal(
            self.price_mixer
                .apply(|p| *p.sell_usd().cents_from_sats(sats.clone()).amount())
                .await?,
        );
        Ok(self.fee_calculator.increase_by_immediate_fee(cents).ceil())
    }

    #[instrument(skip_all, fields(correlation_id, amount = %sats.amount()), ret, err)]
    pub async fn get_cents_from_sats_for_future_buy(
        &self,
        sats: Sats,
    ) -> Result<UsdCents, PriceAppError> {
        let cents = UsdCents::from_decimal(
            self.price_mixer
                .apply(|p| *p.buy_usd().cents_from_sats(sats.clone()).amount())
                .await?,
        );
        Ok(self.fee_calculator.decrease_by_delayed_fee(cents).floor())
    }

    #[instrument(skip_all, fields(correlation_id, amount = %sats.amount()), ret, err)]
    pub async fn get_cents_from_sats_for_future_sell(
        &self,
        sats: Sats,
    ) -> Result<UsdCents, PriceAppError> {
        let cents = UsdCents::from_decimal(
            self.price_mixer
                .apply(|p| *p.sell_usd().cents_from_sats(sats.clone()).amount())
                .await?,
        );
        Ok(self.fee_calculator.increase_by_delayed_fee(cents).ceil())
    }

    #[instrument(skip_all, fields(correlation_id, amount = %cents.amount()), ret, err)]
    pub async fn get_sats_from_cents_for_immediate_buy(
        &self,
        cents: UsdCents,
    ) -> Result<Sats, PriceAppError> {
        let sats = Sats::from_decimal(
            self.price_mixer
                .apply(|p| *p.buy_usd().sats_from_cents(cents.clone()).amount())
                .await?,
        );
        Ok(self.fee_calculator.increase_by_immediate_fee(sats).ceil())
    }

    #[instrument(skip_all, fields(correlation_id, amount = %cents.amount()), ret, err)]
    pub async fn get_sats_from_cents_for_immediate_sell(
        &self,
        cents: UsdCents,
    ) -> Result<Sats, PriceAppError> {
        let sats = Sats::from_decimal(
            self.price_mixer
                .apply(|p| *p.sell_usd().sats_from_cents(cents.clone()).amount())
                .await?,
        );

        Ok(self.fee_calculator.decrease_by_immediate_fee(sats).floor())
    }

    #[instrument(skip_all, fields(correlation_id, amount = %cents.amount()), ret, err)]
    pub async fn get_sats_from_cents_for_future_buy(
        &self,
        cents: UsdCents,
    ) -> Result<Sats, PriceAppError> {
        let sats = Sats::from_decimal(
            self.price_mixer
                .apply(|p| *p.buy_usd().sats_from_cents(cents.clone()).amount())
                .await?,
        );

        Ok(self.fee_calculator.increase_by_delayed_fee(sats).ceil())
    }

    #[instrument(skip_all, fields(correlation_id, amount = %cents.amount()), ret, err)]
    pub async fn get_sats_from_cents_for_future_sell(
        &self,
        cents: UsdCents,
    ) -> Result<Sats, PriceAppError> {
        let sats = Sats::from_decimal(
            self.price_mixer
                .apply(|p| *p.sell_usd().sats_from_cents(cents.clone()).amount())
                .await?,
        );
        Ok(self.fee_calculator.decrease_by_delayed_fee(sats).floor())
    }

    #[instrument(skip_all, fields(correlation_id), ret, err)]
    pub async fn get_cents_per_sat_exchange_mid_rate(&self) -> Result<f64, PriceAppError> {
        let cents_per_sat = self
            .price_mixer
            .apply(|p| *p.mid_price_of_one_sat().amount())
            .await?;
        Ok(f64::try_from(cents_per_sat)?)
    }
}
