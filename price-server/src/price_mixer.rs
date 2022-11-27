use async_trait::async_trait;
use rust_decimal::Decimal;
use thiserror::Error;

use crate::currency::VolumePicker;
use shared::time::*;
use std::collections::HashMap;

pub trait SidePicker {
    fn buy_usd<'a>(&'a self) -> Box<dyn VolumePicker + 'a>;
    fn sell_usd<'a>(&'a self) -> Box<dyn VolumePicker + 'a>;
    fn mid_price_of_one_sat<'a>(&'a self) -> Box<dyn VolumePicker + 'a>;
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
        for (provider, weight) in self.providers.values() {
            let side_picker = match provider.latest().await {
                Ok(side_picker) => side_picker,
                Err(_) => continue,
            };
            println!("call_picker");
            total_weights += weight;
            total += f(&side_picker) * weight;
        }
        Ok(total / total_weights)
    }
}

#[derive(Error, Debug)]
pub enum ExchangePriceCacheError {
    #[error("StalePrice: last update was at {0}")]
    StalePrice(TimeStamp),
    #[error("No price data available")]
    NoPriceAvailable,
}

mod tests {
    pub use std::collections::HashMap;

    pub use chrono::Duration;
    pub use rust_decimal::Decimal;

    pub use super::PriceMixer;
    pub use super::PriceProvider;
    pub use crate::currency::UsdCents;
    pub use crate::{
        currency::{Sats, VolumePicker},
        exchange_tick_cache::ExchangeTickCache,
    };

    #[tokio::test]
    async fn test_price_mixer() {
        let mut providers: HashMap<String, (Box<dyn PriceProvider + Sync + Send>, Decimal)> =
            HashMap::new();
        let cache = ExchangeTickCache::new(Duration::seconds(30));
        providers.insert("okex".to_string(), (Box::new(cache), Decimal::from(1)));
        let price_mixer = PriceMixer::new(providers);
        //cache.apply_update(payload, id)

        let usd = UsdCents::from_decimal(Decimal::ONE);

        let price = price_mixer
            .apply(|p| p.sell_usd().sats_from_cents(usd.clone()).amount().clone())
            .await
            .unwrap();
        println!("price: {}", price);
    }
}
