use std::env;
use thiserror::Error;

use rust_decimal::prelude::*;

#[derive(Error, Debug)]
pub enum FeeCalculatorError {
    #[error("Error in getting config {0}")]
    FeeConfigError(env::VarError),
    #[error("Error in calculating fee")]
    CalculationError,
    #[error("No trading amount supplied to calculate fees")]
    NoTradingAmountAvailableError,
}

pub struct FeeCalculator {
    base_fee_rate: Decimal,
    immediate_fee_rate: Decimal,
    delay_fee_rate: Decimal,
}

impl Default for FeeCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl FeeCalculator {
    pub fn new() -> Self {
        let base_fee = env::var("BASE_FEE_RATE").unwrap_or_else(|_| "0.005".to_string());
        let base_fee_rate = Decimal::from_str(&base_fee).expect("Error converting base fee");

        let immediate_fee = env::var("IMMEDIATE_FEE_RATE").unwrap_or_else(|_| "0.002".to_string());
        let immediate_fee_rate =
            Decimal::from_str(&immediate_fee).expect("Error converting immediate fee");

        let delay_fee = env::var("DELAYED_FEE_RATE").unwrap_or_else(|_| "0.0025".to_string());
        let delay_fee_rate = Decimal::from_str(&delay_fee).expect("Error converting delay fee");

        Self {
            base_fee_rate,
            immediate_fee_rate,
            delay_fee_rate,
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
        let fees = FeeCalculator::new();

        let immediate_fee = Decimal::new(993, 3);
        let delay_fee = Decimal::new(9925, 4);
        assert_eq!(fees.apply_immediate_fee(), immediate_fee);
        assert_eq!(fees.apply_delayed_fee(), delay_fee);
    }
}
