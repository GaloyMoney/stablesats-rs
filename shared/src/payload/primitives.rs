use thiserror::Error;

use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};

use crate::currency::*;

crate::string_wrapper! { ExchangeIdRaw }
crate::string_wrapper! { InstrumentIdRaw }
crate::string_wrapper! { CurrencyRaw }

const PRICE_IN_CENTS_PRECISION: u32 = 12;

#[derive(Error, Debug)]
pub enum ConvertU64ToF64Error {
    #[error("Error converting u64 to f64 type")]
    ConversionError,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceRatioRaw {
    pub numerator_unit: CurrencyRaw,
    pub denominator_unit: CurrencyRaw,
    pub(super) offset: u32,
    pub(super) base: Decimal,
}
impl PriceRatioRaw {
    pub fn from_one_btc_in_usd_price(price: Decimal) -> Self {
        let price_in_cents = price * Decimal::from(100);
        let price_with_precision =
            price_in_cents * Decimal::from(10_u64.pow(PRICE_IN_CENTS_PRECISION));
        let base = price_with_precision / Decimal::from(100_000_000);
        Self {
            numerator_unit: CurrencyRaw::from(UsdCents::code()),
            denominator_unit: CurrencyRaw::from(Sats::code()),
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

pub struct ConvertU64ToF64 {
    pub amount: u64,
}

impl ConvertU64ToF64 {
    pub fn convert(&self) -> Result<f64, ConvertU64ToF64Error> {
        match f64::from_u64(self.amount) {
            Some(result) => Ok(result),
            None => Err(ConvertU64ToF64Error::ConversionError),
        }
    }
}

#[cfg(test)]
mod tests {
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
        let rate = ratio.numerator_amount();
        assert_eq!(rate.to_string(), "0.000000000123".to_string());
    }

    #[test]
    fn from_usd_btc_price() -> anyhow::Result<()> {
        let amount = "9999.99".parse::<Decimal>()?;
        let ratio = PriceRatioRaw::from_one_btc_in_usd_price(amount);

        assert_eq!(ratio.numerator_unit, CurrencyRaw::from("USD_CENT"));
        assert_eq!(ratio.denominator_unit, CurrencyRaw::from("SATOSHI"));

        assert_eq!(ratio.offset, 12);
        assert_eq!(&ratio.base.to_string(), "9999990000");
        assert_eq!(&ratio.numerator_amount().to_string(), "0.00999999");

        Ok(())
    }

    #[test]
    fn convert_u64_to_f64() {
        let amount_to_convert = ConvertU64ToF64 { amount: 200 }.convert().unwrap();

        assert_eq!(amount_to_convert, f64::from(200));

        assert_ne!(amount_to_convert, f64::from(100));
    }
}
