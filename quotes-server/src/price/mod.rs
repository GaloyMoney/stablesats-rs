mod error;
mod fee;
mod mixer;
mod tick_converter;
mod traits;

use crate::currency::*;

pub use error::*;
pub use fee::*;
pub use mixer::*;
pub use tick_converter::*;
pub use traits::*;

pub struct PriceCalculator {
    fee_calculator: FeeCalculator,
    price_mixer: PriceMixer,
}

impl PriceCalculator {
    pub fn new(fee_cfg: FeeCalculatorConfig, price_mixer: PriceMixer) -> Self {
        Self {
            fee_calculator: FeeCalculator::new(fee_cfg),
            price_mixer,
        }
    }

    pub async fn cents_from_sats_for_buy(
        &self,
        sats: Satoshis,
        immediate_execution: bool,
    ) -> Result<UsdCents, ExchangePriceCacheError> {
        let cents = self
            .price_mixer
            .apply(|p| p.buy_usd().cents_from_sats(sats.clone()))
            .await?;
        Ok(self
            .fee_calculator
            .decrease_by_fee(immediate_execution, cents)
            .floor())
    }
}
