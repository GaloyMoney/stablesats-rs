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
    pub tick: TickerChannelData,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn bitfinex_price_tick() {
        let response_text =
            "[225440,[21099,66.42015405,21101,36.16639035,-3,-0.0001,21101,2780.23882622,21469,20639]]";
        let details = serde_json::from_str::<BitfinexPriceTick>(response_text).unwrap();
        dbg!(details.clone());
        assert_eq!(details.tick.bid, dec!(21099));
    }
}
