use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::ops::Mul;

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

    pub fn increase_by_fee<T: Mul<Decimal>>(
        &self,
        immediate_execution: bool,
        currency: T,
    ) -> <T as Mul<Decimal>>::Output {
        currency
            * (dec!(1)
                + if immediate_execution {
                    self.immediate_rate
                } else {
                    self.delayed_rate
                })
    }

    pub fn decrease_by_fee<T: Mul<Decimal>>(
        &self,
        immediate_execution: bool,
        currency: T,
    ) -> <T as Mul<Decimal>>::Output {
        currency
            * (dec!(1)
                - if immediate_execution {
                    self.immediate_rate
                } else {
                    self.delayed_rate
                })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FeeCalculatorConfig {
    #[serde(default = "default_base_fee_rate")]
    pub base_fee_rate: Decimal,
    #[serde(default = "default_immediate_fee_rate")]
    pub immediate_fee_rate: Decimal,
    #[serde(default = "default_delayed_fee_rate")]
    pub delayed_fee_rate: Decimal,
}

fn default_base_fee_rate() -> Decimal {
    dec!(0.0005)
}

fn default_immediate_fee_rate() -> Decimal {
    dec!(0.0005)
}

fn default_delayed_fee_rate() -> Decimal {
    dec!(0.0007)
}

impl Default for FeeCalculatorConfig {
    fn default() -> Self {
        Self {
            base_fee_rate: default_base_fee_rate(),
            immediate_fee_rate: default_immediate_fee_rate(),
            delayed_fee_rate: default_delayed_fee_rate(),
        }
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

        let usd_in = UsdCents::from(dec!(10_000));
        assert_eq!(
            fees.decrease_by_fee(true, usd_in.clone()),
            UsdCents::from(Decimal::from(10_000 - 110))
        );
        assert_eq!(
            fees.decrease_by_fee(false, usd_in.clone()),
            UsdCents::from(Decimal::from(10_000 - 1010))
        );
        assert_eq!(
            fees.increase_by_fee(true, usd_in.clone()),
            UsdCents::from(Decimal::from(10_000 + 110))
        );
        assert_eq!(
            fees.increase_by_fee(false, usd_in),
            UsdCents::from(Decimal::from(10_000 + 1010))
        );
    }

    #[test]
    fn config_defaults() {
        assert_eq!(
            default_base_fee_rate(),
            Decimal::from_str_exact("0.0005").unwrap()
        );
        assert_eq!(
            default_immediate_fee_rate(),
            Decimal::from_str_exact("0.0005").unwrap()
        );
        assert_eq!(
            default_delayed_fee_rate(),
            Decimal::from_str_exact("0.0007").unwrap()
        );
    }
}
