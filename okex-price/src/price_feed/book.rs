use crate::ChannelArgs;
use rust_decimal::Decimal;
use serde::Deserialize;
use shared::time::TimeStampMilliStr;

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OrderBookAction {
    Snapshot,
    Update,
}

#[derive(Debug, Deserialize)]
pub struct CheckSum(String);

#[derive(Debug, Deserialize)]
pub struct OrderBookPriceData {
    pub price: Decimal,
    pub quantity: u32,
    pub zero: u8,
    pub orders: u32,
}

#[derive(Debug, Deserialize)]
pub struct OrderBookChannelData {
    pub asks: Vec<Vec<OrderBookPriceData>>,
    pub bids: Vec<Vec<OrderBookPriceData>>,
    pub ts: TimeStampMilliStr,
    pub checksum: i64,
}

#[derive(Debug, Deserialize)]
pub struct OkexOrderBook {
    pub arg: ChannelArgs,
    pub action: OrderBookAction,
    pub data: Vec<OrderBookChannelData>,
}

#[derive(Debug, Deserialize)]
pub struct Connect {
    pub event: String,
    pub arg: ChannelArgs,
}
