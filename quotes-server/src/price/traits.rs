use async_trait::async_trait;

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
