use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

crate::string_wrapper! { ExchangeIdRaw }
crate::string_wrapper! { InstrumentIdRaw }
crate::string_wrapper! { CurrencyRaw }
crate::decimal_wrapper! { QuantityRaw }
crate::decimal_wrapper! { PriceRaw }

crate::abs_decimal_wrapper! { SyntheticCentLiability }
crate::decimal_wrapper! { SyntheticCentExposure }

pub const USD_CENT_UNIT_NAME: &str = "USD_CENT";
pub const SATOSHI_UNIT_NAME: &str = "SATOSHI";

const PRICE_IN_CENTS_PRECISION: u32 = 12;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceRatioRaw {
    pub numerator_unit: CurrencyRaw,
    pub denominator_unit: CurrencyRaw,
    pub(super) offset: u32,
    pub(super) base: Decimal,
}
impl PriceRatioRaw {
    pub fn from_one_btc_in_usd_price(price: Decimal) -> Self {
        let price_in_cents = price * dec!(100);
        let price_with_precision =
            price_in_cents * Decimal::from(10_u64.pow(PRICE_IN_CENTS_PRECISION));
        let base = price_with_precision / dec!(100_000_000);
        Self {
            numerator_unit: CurrencyRaw::from(USD_CENT_UNIT_NAME),
            denominator_unit: CurrencyRaw::from(SATOSHI_UNIT_NAME),
            offset: PRICE_IN_CENTS_PRECISION,
            base: base.trunc(),
        }
    }

    pub fn numerator_amount(&self) -> Decimal {
        let mut ret = self.base;
        ret.set_scale(self.offset).expect("failed to set scale");
        ret.normalize()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CheckSumRaw(i32);
impl From<i32> for CheckSumRaw {
    fn from(cs: i32) -> Self {
        Self(cs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn serialize_ratio() {
        let ratio = PriceRatioRaw {
            numerator_unit: CurrencyRaw("USD".to_string()),
            denominator_unit: CurrencyRaw("BTC".to_string()),
            offset: 2,
            base: dec!(123),
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
            base: dec!(123),
        };
        let rate = ratio.numerator_amount();
        assert_eq!(rate.to_string(), "0.000000000123".to_string());
    }

    #[test]
    fn from_usd_btc_price() -> anyhow::Result<()> {
        let amount = dec!(9999.99);
        let ratio = PriceRatioRaw::from_one_btc_in_usd_price(amount);

        assert_eq!(ratio.numerator_unit, CurrencyRaw::from("USD_CENT"));
        assert_eq!(ratio.denominator_unit, CurrencyRaw::from("SATOSHI"));

        assert_eq!(ratio.offset, 12);
        assert_eq!(&ratio.base.to_string(), "9999990000");
        assert_eq!(&ratio.numerator_amount().to_string(), "0.00999999");

        Ok(())
    }
}
