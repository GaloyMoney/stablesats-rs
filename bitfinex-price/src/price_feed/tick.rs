use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Clone, Deserialize, Debug)]
pub struct TickerChannelData {
    pub bid: Decimal,
    pub bid_size: Decimal,
    pub ask: Decimal,
    pub ask_size: Decimal,
    pub daily_change: Decimal,
    pub daily_change_relative: Decimal,
    pub last_price: Decimal,
    pub volume: Decimal,
    pub high: Decimal,
    pub low: Decimal,
}

#[derive(Clone, Deserialize, Debug)]
pub struct BitfinexPriceTick {
    pub channel_id: u64,
    pub heartbeat: Option<String>,
    pub tick: Option<TickerChannelData>,
}
