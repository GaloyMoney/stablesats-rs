use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use shared::time::*;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChannelArgs {
    pub channel: String,
    pub inst_id: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TickersChannelData {
    pub ask_px: Decimal,
    pub bid_px: Decimal,
    pub ts: TimeStampMilliStr,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct OkexPriceTick {
    pub arg: ChannelArgs,
    pub data: Vec<TickersChannelData>,
}
