use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ExchangeIdRaw(String);

#[derive(Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InstrumentIdRaw(String);

#[derive(Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CurrencyRaw(String);

#[derive(Clone, Serialize, Deserialize)]
pub struct PriceRatioRaw {
    pub numerator_unit: CurrencyRaw,
    pub denominator_unit: CurrencyRaw,
    pub offset: u16,
    pub base: Decimal,
    pub formatted_amount: String,
}

#[cfg(test)]
mod test {
    #[test]
    fn serialize_ratio() {
        use super::*;
        let ratio = PriceRatioRaw {
            numerator_unit: CurrencyRaw("USD".to_string()),
            denominator_unit: CurrencyRaw("BTC".to_string()),
            offset: 2,
            base: Decimal::new(123, 0),
            formatted_amount: "1.23".to_string(),
        };
        let serialized = serde_json::to_string(&ratio).unwrap();

        assert_eq!(
            serialized,
            r#"{"numerator_unit":"USD","denominator_unit":"BTC","offset":2,"base":"123","formatted_amount":"1.23"}"#
        );
    }
}
