use shared::payload::*;

use super::*;

impl TryFrom<PriceRatioRaw> for UsdCents {
    type Error = CurrencyError;

    fn try_from(ratio: PriceRatioRaw) -> Result<Self, Self::Error> {
        if ratio.numerator_unit.as_str() != UsdCents::code() {
            return Err(CurrencyError::Conversion(
                ratio.numerator_unit.to_string(),
                UsdCents::code(),
            ));
        }
        Ok(UsdCents::from(ratio.numerator_amount()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn convert_to_usd_cents() {
        let ratio = PriceRatioRaw::from_one_btc_in_usd_price(dec!(99999));
        let price_of_one_sat = UsdCents::try_from(ratio).unwrap();
        assert_eq!(
            price_of_one_sat.amount().to_string(),
            "0.099999".to_string()
        );
    }
}
