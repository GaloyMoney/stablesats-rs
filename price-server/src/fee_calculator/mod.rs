mod config;

use rust_decimal::prelude::*;

use shared::currency::*;

pub use config::*;

pub struct FeeCalculator {
    base_fee_rate: Decimal,
    immediate_fee_rate: Decimal,
    delayed_fee_rate: Decimal,
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
            base_fee_rate,
            immediate_fee_rate,
            delayed_fee_rate,
        }
    }

    pub fn apply_immediate_fee(&self, cents: UsdCents) -> UsdCents {
        cents * (Decimal::from(1) - (self.base_fee_rate + self.immediate_fee_rate))
    }

    pub fn apply_delayed_fee(&self, cents: UsdCents) -> UsdCents {
        cents * (Decimal::from(1) - (self.base_fee_rate + self.delayed_fee_rate))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fee_calculator() {
        let fees = FeeCalculator::new(FeeCalculatorConfig {
            base_fee_rate: "0.001".to_string().parse::<Decimal>().unwrap(),
            immediate_fee_rate: "0.01".to_string().parse::<Decimal>().unwrap(),
            delayed_fee_rate: "0.1".to_string().parse::<Decimal>().unwrap(),
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
