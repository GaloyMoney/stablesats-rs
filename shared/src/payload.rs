use serde::{Deserialize, Serialize};

use super::exchange::*;
use super::pubsub::*;
use super::time::*;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceMessagePayload {
    pub timestamp: TimeStamp,
    pub exchange: ExchangeIdRaw,
    pub instrument_id: InstrumentIdRaw,
    pub ask_price: PriceRatioRaw,
    pub bid_price: PriceRatioRaw,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OkexBtcUsdSwapPricePayload(pub PriceMessagePayload);

crate::payload! { OkexBtcUsdSwapPricePayload, "price.okex.btc-usd-swap" }
