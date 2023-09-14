use crate::currency::*;

use super::traits::VolumePicker;

pub struct TickCurrencyConverter<'a> {
    price_of_one_sat: &'a UsdCents,
}

impl<'a> TickCurrencyConverter<'a> {
    pub fn new(price_of_one_sat: &'a UsdCents) -> Self {
        Self { price_of_one_sat }
    }
}

impl<'a> VolumePicker for TickCurrencyConverter<'a> {
    fn cents_from_sats(&self, volume: Satoshis) -> UsdCents {
        UsdCents::from(volume.amount() * self.price_of_one_sat.amount())
    }

    fn sats_from_cents(&self, volume: UsdCents) -> Satoshis {
        Satoshis::from(volume.amount() / self.price_of_one_sat.amount())
    }
}
