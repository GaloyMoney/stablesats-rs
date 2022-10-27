use rust_decimal::Decimal;

use crate::currency::{Sats, UsdCents};

#[derive(Debug)]
pub struct PriceConverter {
    weighted_price_of_one_sat: Decimal,
}
impl PriceConverter {
    pub fn new(price: Decimal) -> Self {
        Self {
            weighted_price_of_one_sat: price,
        }
    }

    pub fn cents_from_sats(&self, sats: Sats) -> UsdCents {
        UsdCents::from_decimal(sats.amount() * self.weighted_price_of_one_sat)
    }

    pub fn sats_from_cents(&self, cents: UsdCents) -> Sats {
        Sats::from_decimal(cents.amount() / self.weighted_price_of_one_sat)
    }
}
