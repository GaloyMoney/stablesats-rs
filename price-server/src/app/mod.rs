mod config;

use futures::stream::StreamExt;
use rust_decimal::Decimal;
use tracing::{instrument, trace_span, Instrument};

use shared::{
    health::HealthCheckTrigger,
    payload::{PriceStreamPayload, OKEX_EXCHANGE_ID},
    pubsub::*,
};

use crate::{cache_config::ExchangePriceCacheConfig, price_mixer::PriceMixer, OrderBookCache};

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
        exchange_weights: ExchangeWeights,
    ) -> Result<Self, PriceAppError> {
        let health_subscriber = subscriber.resubscribe();
        tokio::spawn(async move {
            while let Some(check) = health_check_trigger.next().await {
                let _ = check.send(
                    health_subscriber
                        .healthy(health_check_cfg.unhealthy_msg_interval_price)
                        .await,
                );
            }
        });

        let mut price_mixer = PriceMixer::new();

        if let Some(weight) = exchange_weights.okex {
            if weight > Decimal::ZERO {
                let okex_order_book_cache = OrderBookCache::new(price_cache_config.clone());
                Self::subscribe_okex(subscriber.resubscribe(), okex_order_book_cache.clone())
                    .await?;
                price_mixer.add_provider(OKEX_EXCHANGE_ID, okex_order_book_cache, weight);
            }
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
        order_book_cache: OrderBookCache,
    ) -> Result<(), PriceAppError> {
        tokio::spawn(async move {
            while let Some(msg) = subscriber.next().await {
                if let PriceStreamPayload::OkexBtcUsdSwapOrderBookPayload(price_msg) = msg.payload {
                    let span = trace_span!(
                        "price_server.okex_order_book_received",
                        message_type = %msg.payload_type,
                        correlation_id = %msg.meta.correlation_id
                    );
                    shared::tracing::inject_tracing_data(&span, &msg.meta.tracing_data);
                    async {
                        order_book_cache.apply_update(price_msg).await;
                    }
                    .instrument(span)
                    .await;
                }
            }
        });

        Ok(())
    }

    #[instrument(name = "price_server.get_cents_from_sats_for_immediate_buy", skip_all, fields(correlation_id, amount = %sats.amount()), ret, err)]
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

    #[instrument(name = "price_server.get_cents_from_sats_for_immediate_sell", skip_all, fields(correlation_id, amount = %sats.amount()), ret, err)]
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

    #[instrument(name = "price_server.get_cents_from_sats_for_future_buy", skip_all, fields(correlation_id, amount = %sats.amount()), ret, err)]
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

    #[instrument(name = "price_server.get_cents_from_sats_for_future_sell", skip_all, fields(correlation_id, amount = %sats.amount()), ret, err)]
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

    #[instrument(name = "price_server.get_sats_from_cents_for_immediate_buy", skip_all, fields(correlation_id, amount = %cents.amount()), ret, err)]
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

    #[instrument(name = "price_server.get_sats_from_cents_for_immediate_sell", skip_all, fields(correlation_id, amount = %cents.amount()), ret, err)]
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

    #[instrument(name = "price_server.get_sats_from_cents_for_future_buy", skip_all, fields(correlation_id, amount = %cents.amount()), ret, err)]
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

    #[instrument(name = "price_server.get_sats_from_cents_for_future_sell", skip_all, fields(correlation_id, amount = %cents.amount()), ret, err)]
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

    #[instrument(
        name = "price_server.get_cents_per_sat_exchange_mid_rate",
        skip_all,
        fields(correlation_id),
        ret,
        err
    )]
    pub async fn get_cents_per_sat_exchange_mid_rate(&self) -> Result<f64, PriceAppError> {
        let cents_per_sat = self
            .price_mixer
            .apply(|p| *p.mid_price_of_one_sat().amount())
            .await?;
        Ok(f64::try_from(cents_per_sat)?)
    }
}
