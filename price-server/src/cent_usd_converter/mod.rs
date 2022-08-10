mod constants;

use rust_decimal::Decimal;
use shared::currency::*;

use constants::CENTS_PER_USD;

pub struct CentUsdConverter {
    pub cents: UsdCents,
}

impl CentUsdConverter {
    pub fn convert(&self) -> Usd {
        let result = self.cents.amount() / Decimal::new(CENTS_PER_USD, 0);

        Usd::from_decimal(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_cents_to_usd() {
        let cents = UsdCents::from_major(1000);

        let converter = CentUsdConverter { cents };

        assert_eq!(&Decimal::new(10, 0), converter.convert().amount());
    }
}
