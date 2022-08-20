use rust_decimal::Decimal;
use serde::Deserialize;

use shared::time::*;

#[derive(Clone, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChannelArgs {
    pub channel: String,
    pub inst_id: String,
}

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
