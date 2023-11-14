use rust_decimal::Decimal;
use std::collections::HashMap;

use super::{error::ExchangePriceCacheError, traits::*};

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

    pub async fn apply<R: ExchangeableCurrency>(
        &self,
        f: impl Fn(&Box<dyn SidePicker>) -> R,
    ) -> Result<R, ExchangePriceCacheError> {
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
            total += f(&side_picker).into() * weight;
        }

        if total_weights > Decimal::ZERO {
            Ok(R::from(total / total_weights))
        } else {
            Err(prev_error.unwrap_or(ExchangePriceCacheError::NoPriceAvailable))
        }
    }
}

impl Default for PriceMixer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    pub use std::collections::HashMap;

    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    use crate::{currency::*, price::*};

    #[tokio::test]
    async fn test_price_mixer() -> anyhow::Result<()> {
        let mut price_mixer = PriceMixer::new();
        let one = DummyProvider::new(UsdCents::from(Decimal::ONE), UsdCents::from(Decimal::ONE));
        price_mixer.add_provider("one", one, Decimal::ONE);
        let two = DummyProvider::new(UsdCents::from(Decimal::TWO), UsdCents::from(Decimal::TWO));
        price_mixer.add_provider("two", two, Decimal::ONE);

        let price = price_mixer
            .apply(|p| p.sell_usd().sats_from_cents(UsdCents::from(dec!(4))))
            .await
            .expect("Price should be available");
        assert_eq!(dec!(3), *price.amount());
        Ok(())
    }
}
