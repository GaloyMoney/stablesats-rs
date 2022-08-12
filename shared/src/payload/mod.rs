mod constants;
mod convert;
mod primitives;

use serde::{Deserialize, Serialize};

use super::pubsub::*;
use super::time::*;

pub use constants::*;
pub use primitives::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceMessagePayload {
    pub timestamp: TimeStamp,
    pub exchange: ExchangeIdRaw,
    pub instrument_id: InstrumentIdRaw,
    pub ask_price: PriceRatioRaw,
    pub bid_price: PriceRatioRaw,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OkexBtcUsdSwapPricePayload(pub PriceMessagePayload);
impl From<OkexBtcUsdSwapPricePayload> for PriceMessagePayload {
    fn from(payload: OkexBtcUsdSwapPricePayload) -> Self {
        payload.0
    }
}
impl std::ops::Deref for OkexBtcUsdSwapPricePayload {
    type Target = PriceMessagePayload;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

crate::payload! { OkexBtcUsdSwapPricePayload, "price.okex.btc-usd-swap" }
