use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};

crate::string_wrapper! { ExchangeIdRaw }
crate::string_wrapper! { InstrumentIdRaw }
crate::string_wrapper! { CurrencyRaw }

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceRatioRaw {
    pub numerator_unit: CurrencyRaw,
    pub denominator_unit: CurrencyRaw,
    pub(super) offset: u32,
    pub(super) base: Decimal,
}
impl PriceRatioRaw {
    pub fn rate(&self) -> Decimal {
        let mut ret = self.base.clone();
        let _ = ret.set_scale(self.offset).expect("failed to set scale");
        ret
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn serialize_ratio() {
        let ratio = PriceRatioRaw {
            numerator_unit: CurrencyRaw("USD".to_string()),
            denominator_unit: CurrencyRaw("BTC".to_string()),
            offset: 2,
            base: Decimal::new(123, 0),
        };
        let serialized = serde_json::to_string(&ratio).unwrap();

        assert_eq!(
            serialized,
            r#"{"numeratorUnit":"USD","denominatorUnit":"BTC","offset":2,"base":"123"}"#
        );
    }

    #[test]
    fn amount() {
        let ratio = PriceRatioRaw {
            numerator_unit: CurrencyRaw::from("USD"),
            denominator_unit: CurrencyRaw::from("BTC"),
            offset: 12,
            base: Decimal::new(123, 0),
        };
        let rate = ratio.rate();
        assert_eq!(rate.to_string(), "0.000000000123".to_string());
    }
}
