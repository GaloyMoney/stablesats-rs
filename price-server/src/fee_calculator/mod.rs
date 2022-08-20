mod config;

use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use std::ops::Mul;

pub use config::*;

pub struct FeeCalculator {
    immediate_rate: Decimal,
    delayed_rate: Decimal,
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
            immediate_rate: base_fee_rate + immediate_fee_rate,
            delayed_rate: base_fee_rate + delayed_fee_rate,
        }
    }

    pub fn increase_by_immediate_fee<T: Mul<Decimal>>(
        &self,
        currency: T,
    ) -> <T as Mul<Decimal>>::Output {
        currency * (dec!(1) + self.immediate_rate)
    }

    pub fn increase_by_delayed_fee<T: Mul<Decimal>>(
        &self,
        currency: T,
    ) -> <T as Mul<Decimal>>::Output {
        currency * (dec!(1) + self.delayed_rate)
    }

    pub fn decrease_by_immediate_fee<T: Mul<Decimal>>(
        &self,
        currency: T,
    ) -> <T as Mul<Decimal>>::Output {
        currency * (dec!(1) - self.immediate_rate)
    }

    pub fn decrease_by_delayed_fee<T: Mul<Decimal>>(
        &self,
        currency: T,
    ) -> <T as Mul<Decimal>>::Output {
        currency * (dec!(1) - self.delayed_rate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::currency::*;

    #[test]
    fn fee_calculator() {
        let fees = FeeCalculator::new(FeeCalculatorConfig {
            base_fee_rate: dec!(0.001),
            immediate_fee_rate: dec!(0.01),
            delayed_fee_rate: dec!(0.1),
        });

        let usd_in = UsdCents::from_major(10_000);
        assert_eq!(
            fees.decrease_by_immediate_fee(usd_in.clone()),
            UsdCents::from_major(10_000 - 110)
        );
        assert_eq!(
            fees.decrease_by_delayed_fee(usd_in.clone()),
            UsdCents::from_major(10_000 - 1010)
        );
        assert_eq!(
            fees.increase_by_immediate_fee(usd_in.clone()),
            UsdCents::from_major(10_000 + 110)
        );
        assert_eq!(
            fees.increase_by_delayed_fee(usd_in),
            UsdCents::from_major(10_000 + 1010)
        );
    }
}
