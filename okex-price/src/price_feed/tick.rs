use rust_decimal::Decimal;
use serde::Deserialize;

use shared::time::*;

use crate::ChannelArgs;

#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TickersChannelData {
    pub ask_px: Decimal,
    pub bid_px: Decimal,
    pub ts: TimeStampMilliStr,
}

#[derive(Clone, Deserialize, Debug)]
pub struct OkexPriceTick {
    pub arg: ChannelArgs,
    pub data: Vec<TickersChannelData>,
}
