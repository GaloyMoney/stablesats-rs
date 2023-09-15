mod error;
mod fee;
mod mixer;
mod tick_converter;
mod traits;

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
}
