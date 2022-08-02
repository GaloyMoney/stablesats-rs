use super::primitives::PriceRatioRaw;
use crate::currency::*;

impl TryFrom<PriceRatioRaw> for UsdCents {
    type Error = CurrencyError;

    fn try_from(
        PriceRatioRaw {
            numerator_unit,
            mut base,
            offset,
            ..
        }: PriceRatioRaw,
    ) -> Result<Self, Self::Error> {
        if numerator_unit.0 != UsdCents::code() {
            return Err(CurrencyError::Conversion(
                numerator_unit.to_string(),
                UsdCents::code(),
            ));
        }
        base.set_scale(offset)?;
        Ok(UsdCents::from_decimal(base))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::payload::*;
    use rust_decimal::prelude::*;

    #[test]
    fn convert_to_usd_cents() {
        let ratio = PriceRatioRaw {
            numerator_unit: CurrencyRaw("USD_CENT".to_string()),
            denominator_unit: CurrencyRaw("BTC_SAT".to_string()),
            offset: 12,
            base: Decimal::new(9_999_990_000, 0),
        };
        let price_of_one_sat = UsdCents::try_from(ratio).unwrap();
        assert_eq!(
            price_of_one_sat.amount().to_string(),
            "0.009999990000".to_string()
        );
    }
}
