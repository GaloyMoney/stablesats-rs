use serde::Deserialize;
use shared::time::TimeStampMilliStr;

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChannelArgs {
    pub channel: String,
    pub inst_id: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OrderBookAction {
    Snapshot,
    Update,
}

#[derive(Debug, Deserialize)]
pub struct OrderBookChannelData {
    pub asks: Vec<Vec<String>>,
    pub bids: Vec<Vec<String>>,
    pub ts: TimeStampMilliStr,
    pub checksum: i64,
}

#[derive(Debug, Deserialize)]
pub struct OkexOrderBook {
    pub arg: ChannelArgs,
    pub action: OrderBookAction,
    pub data: Vec<OrderBookChannelData>,
}
