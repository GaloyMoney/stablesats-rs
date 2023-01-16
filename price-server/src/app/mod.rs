mod config;

use futures::stream::StreamExt;
use tracing::{info_span, instrument, Instrument};

use shared::{
    exchanges_config::ExchangeConfigs,
    health::HealthCheckTrigger,
    payload::{PriceStreamPayload, KOLLIDER_EXCHANGE_ID, OKEX_EXCHANGE_ID},
    pubsub::*,
};

use crate::{
    cache_config::ExchangePriceCacheConfig, exchange_tick_cache::ExchangeTickCache,
    price_mixer::PriceMixer,
};
pub use crate::{currency::*, error::*, fee_calculator::*};
pub use config::*;

pub struct PriceApp {
    price_mixer: PriceMixer,
    fee_calculator: FeeCalculator,
}

impl PriceApp {
    pub async fn run(
        mut health_check_trigger: HealthCheckTrigger,
        health_check_cfg: PriceServerHealthCheckConfig,
        fee_calc_cfg: FeeCalculatorConfig,
        subscriber: memory::Subscriber<PriceStreamPayload>,
        price_cache_config: ExchangePriceCacheConfig,
        exchanges_cfg: ExchangeConfigs,
    ) -> Result<Self, PriceAppError> {
        let health_subscriber = subscriber.resubscribe();
        tokio::spawn(async move {
            while let Some(check) = health_check_trigger.next().await {
                check
                    .send(
                        health_subscriber
                            .healthy(health_check_cfg.unhealthy_msg_interval_price)
                            .await,
                    )
                    .expect("Couldn't send response");
            }
        });

        let mut price_mixer = PriceMixer::new();

        if let Some(config) = exchanges_cfg.okex.as_ref() {
            let okex_price_cache = ExchangeTickCache::new(price_cache_config.clone());
            Self::subscribe_okex(subscriber.resubscribe(), okex_price_cache.clone()).await?;
            price_mixer.add_provider(OKEX_EXCHANGE_ID, okex_price_cache, config.weight);
        }

        if let Some(config) = exchanges_cfg.kollider.as_ref() {
            let kollider_price_cache = ExchangeTickCache::new(price_cache_config);
            Self::subscribe_kollider(subscriber, kollider_price_cache.clone()).await?;
            price_mixer.add_provider(KOLLIDER_EXCHANGE_ID, kollider_price_cache, config.weight);
        }

        let fee_calculator = FeeCalculator::new(fee_calc_cfg);
        let app = Self {
            price_mixer,
            fee_calculator,
        };

        Ok(app)
    }

    async fn subscribe_okex(
        mut subscriber: memory::Subscriber<PriceStreamPayload>,
        price_cache: ExchangeTickCache,
    ) -> Result<(), PriceAppError> {
        let _ = tokio::spawn(async move {
            while let Some(msg) = subscriber.next().await {
                if let PriceStreamPayload::OkexBtcSwapPricePayload(price_msg) = msg.payload {
                    let span = info_span!(
                        "okex_price_tick_received",
                        message_type = %msg.payload_type,
                        correlation_id = %msg.meta.correlation_id
                    );
                    shared::tracing::inject_tracing_data(&span, &msg.meta.tracing_data);
                    async {
                        price_cache
                            .apply_update(price_msg, msg.meta.correlation_id)
                            .await;
                    }
                    .instrument(span)
                    .await;
                }
            }
        });

        Ok(())
    }

    async fn subscribe_kollider(
        mut subscriber: memory::Subscriber<PriceStreamPayload>,
        price_cache: ExchangeTickCache,
    ) -> Result<(), PriceAppError> {
        let _ = tokio::spawn(async move {
            while let Some(msg) = subscriber.next().await {
                if let PriceStreamPayload::KolliderBtcUsdSwapPricePayload(price_msg) = msg.payload {
                    let span = info_span!(
                        "kollider_price_tick_received",
                        message_type = %msg.payload_type,
                        correlation_id = %msg.meta.correlation_id
                    );
                    shared::tracing::inject_tracing_data(&span, &msg.meta.tracing_data);
                    async {
                        price_cache
                            .apply_update(price_msg, msg.meta.correlation_id)
                            .await;
                    }
                    .instrument(span)
                    .await;
                }
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
