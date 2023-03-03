use async_trait::async_trait;
use rust_decimal::Decimal;

use crate::{currency::VolumePicker, error::ExchangePriceCacheError};
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
    providers: HashMap<&'static str, (Box<dyn PriceProvider + Sync + Send>, Decimal)>,
}

impl PriceMixer {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn add_provider(
        &mut self,
        exchange_id: &'static str,
        provider: impl PriceProvider + Sync + Send + 'static,
        weight: Decimal,
    ) {
        self.providers
            .insert(exchange_id, (Box::new(provider), weight));
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

        if total_weights > Decimal::ZERO {
            Ok(total / total_weights)
        } else {
            Err(prev_error.unwrap_or(ExchangePriceCacheError::NoPriceAvailable))
        }
    }
}

#[cfg(test)]
mod tests {
    pub use std::collections::HashMap;

    pub use chrono::Duration;
    pub use rust_decimal::Decimal;
    use shared::payload::PriceMessagePayload;
    use shared::pubsub::CorrelationId;
    use shared::time::TimeStamp;

    pub use super::PriceMixer;
    pub use super::PriceProvider;
    pub use crate::currency::UsdCents;
    pub use crate::{
        cache_config::ExchangePriceCacheConfig,
        currency::{Sats, VolumePicker},
        exchange_tick_cache::ExchangeTickCache,
    };
    pub use serde_json::*;

    #[tokio::test]
    async fn test_price_mixer() -> anyhow::Result<(), Error> {
        let cache = ExchangeTickCache::new(ExchangePriceCacheConfig::default());
        let mut price_mixer = PriceMixer::new();
        price_mixer.add_provider("okex", cache.clone(), Decimal::from(1));

        cache
            .apply_update(get_payload(), CorrelationId::new())
            .await;

        let price = price_mixer
            .apply(|p| {
                *p.sell_usd()
                    .sats_from_cents(UsdCents::from_decimal(Decimal::ONE))
                    .amount()
            })
            .await
            .expect("Price should be available");
        assert_ne!(Decimal::ZERO, price);
        Ok(())
    }

    fn get_payload() -> PriceMessagePayload {
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
}
