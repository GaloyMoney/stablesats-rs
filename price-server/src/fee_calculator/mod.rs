mod config;

use rust_decimal::prelude::*;

pub use config::*;

pub struct FeeCalculator {
    base_fee_rate: Decimal,
    immediate_fee_rate: Decimal,
    delay_fee_rate: Decimal,
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
            delay_fee_rate: delayed_fee_rate,
        }
    }

    pub fn apply_immediate_fee(&self) -> Decimal {
        Decimal::new(1, 0) - (self.base_fee_rate + self.immediate_fee_rate)
    }

    pub fn apply_delayed_fee(&self) -> Decimal {
        Decimal::new(1, 0) - (self.base_fee_rate + self.delay_fee_rate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fee_calculator() {
        let fees = FeeCalculator::new(FeeCalculatorConfig::default());

        let immediate_fee = Decimal::new(993, 3);
        let delay_fee = Decimal::new(9925, 4);
        assert_eq!(fees.apply_immediate_fee(), immediate_fee);
        assert_eq!(fees.apply_delayed_fee(), delay_fee);
    }
}
