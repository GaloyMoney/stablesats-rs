mod config;

use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use std::ops::Mul;

pub use config::*;

pub struct FeeCalculator {
    immediate_multiplier: Decimal,
    delayed_multiplier: Decimal,
}

impl FeeCalculator {
    pub fn new(
        FeeCalculatorConfig {
            base_fee_rate,
            immediate_fee_rate,
            delayed_fee_rate,
        }: FeeCalculatorConfig,
    ) -> Self {
        Self {
            immediate_multiplier: (dec!(1) - (base_fee_rate + immediate_fee_rate)),
            delayed_multiplier: (dec!(1) - (base_fee_rate + delayed_fee_rate)),
        }
    }

    pub fn apply_immediate_fee<T: Mul<Decimal>>(&self, currency: T) -> <T as Mul<Decimal>>::Output {
        currency * self.immediate_multiplier
    }

    pub fn apply_delayed_fee<T: Mul<Decimal>>(&self, currency: T) -> <T as Mul<Decimal>>::Output {
        currency * self.delayed_multiplier
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::currency::*;

    #[test]
    fn fee_calculator() {
        let fees = FeeCalculator::new(FeeCalculatorConfig {
            base_fee_rate: dec!(0.001),
            immediate_fee_rate: dec!(0.01),
            delayed_fee_rate: dec!(0.1),
        });

        let usd_in = UsdCents::from_major(10_000);
        assert_eq!(
            fees.apply_immediate_fee(usd_in.clone()),
            UsdCents::from_major(10_000 - 110)
        );
        assert_eq!(
            fees.apply_delayed_fee(usd_in),
            UsdCents::from_major(10_000 - 1010)
        );
    }
}
