use async_trait::async_trait;
use rust_decimal::Decimal;
use thiserror::Error;

use crate::{currency::VolumePicker, error::ExchangePriceCacheError};
use shared::time::*;
use std::collections::HashMap;

use super::currency::*;

pub trait SidePicker {
    fn buy_usd<'a>(&'a self) -> Box<dyn VolumePicker + 'a>;
    fn sell_usd<'a>(&'a self) -> Box<dyn VolumePicker + 'a>;
    fn mid_price_of_one_sat(&self) -> UsdCents;
}

#[async_trait]
pub trait PriceProvider {
    async fn latest(&self) -> Result<Box<dyn SidePicker>, ExchangePriceCacheError>;
}

pub struct PriceMixer {
    providers: HashMap<String, (Box<dyn PriceProvider + Sync + Send>, Decimal)>,
}

impl PriceMixer {
    pub fn new(
        providers: HashMap<String, (Box<dyn PriceProvider + Sync + Send>, Decimal)>,
    ) -> Self {
        Self { providers }
    }

    pub async fn apply(
        &self,
        f: impl Fn(&Box<dyn SidePicker>) -> Decimal,
    ) -> Result<Decimal, ExchangePriceCacheError> {
        let mut total = Decimal::ZERO;
        let mut total_weights = Decimal::ZERO;
        let mut prev_error: Option<ExchangePriceCacheError> = None;
        for (provider, weight) in self.providers.values() {
            let side_picker = match provider.latest().await {
                Ok(side_picker) => side_picker,
                Err(err) => {
                    prev_error = Some(err);
                    continue;
                }
            };
            total_weights += weight;
            total += f(&side_picker) * weight;
        }

        if let Some(prev_error) = prev_error {
            Err(prev_error)
        } else {
            Ok(total / total_weights)
        }
    }
}

#[derive(Error, Debug)]
pub enum PriceTickCacheError {
    #[error("StalePrice: last update was at {0}")]
    StalePrice(TimeStamp),
    #[error("No price data available")]
    NoPriceAvailable,
}

#[cfg(test)]
mod tests {
    pub use std::collections::HashMap;
    use std::fs;

    use rust_decimal_macros::dec;

    pub use chrono::Duration;
    pub use rust_decimal::Decimal;
    use shared::payload::OkexBtcUsdSwapOrderBookPayload;
    use shared::payload::PriceMessagePayload;
    use shared::pubsub::CorrelationId;
    use shared::pubsub::Envelope;
    use shared::time::TimeStamp;

    pub use super::PriceMixer;
    pub use super::PriceProvider;
    pub use crate::currency::UsdCents;
    use crate::OrderBookCache;
    pub use crate::{
        currency::{Sats, VolumePicker},
        exchange_tick_cache::ExchangeTickCache,
    };
    pub use serde_json::*;

    fn get_tick_payload() -> PriceMessagePayload {
        let raw = r#"{
            "exchange": "okex",
            "instrumentId": "BTC-USD-SWAP",
            "timestamp": 1,
            "bidPrice": {
                "numeratorUnit": "USD_CENT",
                "denominatorUnit": "BTC_SAT",
                "offset": 12,
                "base": "1000000000"
            },
            "askPrice": {
                "numeratorUnit": "USD_CENT",
                "denominatorUnit": "BTC_SAT",
                "offset": 12,
                "base": "10000000000"
            }
            }"#;
        let mut price_message_payload =
            serde_json::from_str::<PriceMessagePayload>(raw).expect("Could not parse payload");
        price_message_payload.timestamp = TimeStamp::now();
        price_message_payload
    }

    #[derive(serde::Deserialize)]
    struct Fixture {
        payload: OkexBtcUsdSwapOrderBookPayload,
    }

    fn load_fixture(dataname: &str) -> anyhow::Result<Fixture> {
        let contents = fs::read_to_string(format!(
            "./tests/fixtures/order-book-payload-{}.json",
            dataname
        ))
        .expect("Couldn't load fixtures");
        Ok(serde_json::from_str(&contents)?)
    }

    fn get_order_book_payload() -> Envelope<OkexBtcUsdSwapOrderBookPayload> {
        let order_book_raw = load_fixture("contrived").expect("Failed to load order book payload");
        let mut payload = order_book_raw.payload;
        payload.timestamp = TimeStamp::now();
        let payload = Envelope::new(payload);

        payload
    }

    #[tokio::test]
    async fn test_price_mixer() -> anyhow::Result<(), Error> {
        let mut providers: HashMap<String, (Box<dyn PriceProvider + Sync + Send>, Decimal)> =
            HashMap::new();
        let tick_cache = Box::new(ExchangeTickCache::new(Duration::seconds(3000)));
        let snapshot_cache = Box::new(OrderBookCache::new(Duration::seconds(30000)));

        providers.insert("okex_tick".to_string(), (tick_cache.clone(), dec!(0.5)));
        providers.insert(
            "okex_order_book".to_string(),
            (snapshot_cache.clone(), dec!(0.5)),
        );

        let price_mixer = PriceMixer::new(providers);

        tick_cache
            .apply_update(get_tick_payload(), CorrelationId::new())
            .await;
        snapshot_cache.apply_update(get_order_book_payload()).await;

        let price = price_mixer
            .apply(|p| {
                *p.sell_usd()
                    .sats_from_cents(UsdCents::from_decimal(Decimal::ONE))
                    .amount()
            })
            .await
            .expect("Price should be available");
        assert_ne!(Decimal::ZERO, price);
        assert_eq!(dec!(55), price);
        Ok(())
    }
}
