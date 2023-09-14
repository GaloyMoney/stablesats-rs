use crate::currency::*;

pub trait SidePicker {
    fn buy_usd<'a>(&'a self) -> Box<dyn VolumePicker + 'a>;
    fn sell_usd<'a>(&'a self) -> Box<dyn VolumePicker + 'a>;
    fn mid_price_of_one_sat(&self) -> UsdCents;
}

pub trait VolumePicker {
    fn cents_from_sats(&self, volume: Satoshis) -> UsdCents;
    fn sats_from_cents(&self, volume: UsdCents) -> Satoshis;
}

pub struct CurrencyConverter<'a> {
    price_of_one_sat: &'a UsdCents,
}

impl<'a> VolumePicker for CurrencyConverter<'a> {
    fn cents_from_sats(&self, volume: Satoshis) -> UsdCents {
        UsdCents::from(volume.amount() * self.price_of_one_sat.amount())
    }

    fn sats_from_cents(&self, volume: UsdCents) -> Satoshis {
        Satoshis::from(volume.amount() / self.price_of_one_sat.amount())
    }
}

impl<'a> CurrencyConverter<'a> {
    pub fn new(price_of_one_sat: &'a UsdCents) -> Self {
        Self { price_of_one_sat }
    }
}
