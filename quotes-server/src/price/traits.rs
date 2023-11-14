use async_trait::async_trait;
use rust_decimal::Decimal;

use crate::currency::*;

use super::error::*;

#[async_trait]
pub trait PriceProvider {
    async fn latest(&self) -> Result<Box<dyn SidePicker>, ExchangePriceCacheError>;
}

pub trait SidePicker {
    fn buy_usd<'a>(&'a self) -> Box<dyn VolumePicker + 'a>;
    fn sell_usd<'a>(&'a self) -> Box<dyn VolumePicker + 'a>;
    fn mid_price_of_one_sat(&self) -> UsdCents;
}

pub trait VolumePicker {
    fn cents_from_sats(&self, volume: Satoshis) -> UsdCents;
    fn sats_from_cents(&self, volume: UsdCents) -> Satoshis;
}

pub trait ExchangeableCurrency: Into<Decimal> + From<Decimal> {}
impl ExchangeableCurrency for Satoshis {}
impl ExchangeableCurrency for UsdCents {}

#[cfg(test)]
mod dummy_impls {
    use rust_decimal::Decimal;

    use super::*;

    pub struct DummyProvider {
        ask_price_of_one_sat: Decimal,
        sell_price_of_one_sat: Decimal,
    }
    impl DummyProvider {
        pub fn new(ask_price_of_one_sat: UsdCents, sell_price_of_one_sat: UsdCents) -> Self {
            Self {
                ask_price_of_one_sat: ask_price_of_one_sat.into(),
                sell_price_of_one_sat: sell_price_of_one_sat.into(),
            }
        }
    }
    #[async_trait]
    impl PriceProvider for DummyProvider {
        async fn latest(&self) -> Result<Box<dyn SidePicker>, ExchangePriceCacheError> {
            Ok(Box::new(DummySidePicker {
                ask_price_of_one_sat: self.ask_price_of_one_sat,
                sell_price_of_one_sat: self.sell_price_of_one_sat,
            }))
        }
    }

    pub struct DummySidePicker {
        ask_price_of_one_sat: Decimal,
        sell_price_of_one_sat: Decimal,
    }
    impl SidePicker for DummySidePicker {
        fn buy_usd<'a>(&'a self) -> Box<dyn VolumePicker + 'a> {
            Box::new(DummyVolumePicker {
                price_of_one_sat: &self.ask_price_of_one_sat,
            })
        }
        fn sell_usd<'a>(&'a self) -> Box<dyn VolumePicker + 'a> {
            Box::new(DummyVolumePicker {
                price_of_one_sat: &self.sell_price_of_one_sat,
            })
        }
        fn mid_price_of_one_sat(&self) -> UsdCents {
            UsdCents::from(Decimal::ONE)
        }
    }
    pub struct DummyVolumePicker<'a> {
        price_of_one_sat: &'a Decimal,
    }
    impl<'a> VolumePicker for DummyVolumePicker<'a> {
        fn cents_from_sats(&self, volume: Satoshis) -> UsdCents {
            (Decimal::from(volume) * self.price_of_one_sat).into()
        }
        fn sats_from_cents(&self, volume: UsdCents) -> Satoshis {
            (Decimal::from(volume) / self.price_of_one_sat).into()
        }
    }
}

#[cfg(test)]
pub use dummy_impls::*;
