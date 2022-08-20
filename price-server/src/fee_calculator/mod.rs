mod config;

use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use std::ops::Mul;

pub use config::*;

pub struct FeeCalculator {
    immediate_buy_multiplier: Decimal,
    delayed_buy_multiplier: Decimal,
    immediate_sell_multiplier: Decimal,
    delayed_sell_multiplier: Decimal,
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
            immediate_buy_multiplier: (dec!(1) - (base_fee_rate + immediate_fee_rate)),
            delayed_buy_multiplier: (dec!(1) - (base_fee_rate + delayed_fee_rate)),
            immediate_sell_multiplier: dec!(1) + base_fee_rate + immediate_fee_rate,
            delayed_sell_multiplier: dec!(1) + base_fee_rate + delayed_fee_rate,
        }
    }

    pub fn apply_immediate_buy_fee<T: Mul<Decimal>>(
        &self,
        currency: T,
    ) -> <T as Mul<Decimal>>::Output {
        currency * self.immediate_buy_multiplier
    }

    pub fn apply_delayed_buy_fee<T: Mul<Decimal>>(
        &self,
        currency: T,
    ) -> <T as Mul<Decimal>>::Output {
        currency * self.delayed_buy_multiplier
    }

    pub fn apply_immediate_sell_fee<T: Mul<Decimal>>(
        &self,
        currency: T,
    ) -> <T as Mul<Decimal>>::Output {
        currency * self.immediate_sell_multiplier
    }

    pub fn apply_delayed_sell_fee<T: Mul<Decimal>>(
        &self,
        currency: T,
    ) -> <T as Mul<Decimal>>::Output {
        currency * self.delayed_sell_multiplier
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
            fees.apply_immediate_buy_fee(usd_in.clone()),
            UsdCents::from_major(10_000 - 110)
        );
        assert_eq!(
            fees.apply_delayed_buy_fee(usd_in),
            UsdCents::from_major(10_000 - 1010)
        );
    }
}
