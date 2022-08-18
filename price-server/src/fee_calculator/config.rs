use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

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

    #[test]
    fn defaults() {
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
