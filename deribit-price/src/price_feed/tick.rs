use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Clone, Deserialize, Debug)]
pub struct TickerChannelData {
    pub timestamp: u64,
    pub instrument_name: String,
    pub best_bid_price: Decimal,
    pub best_bid_amount: Decimal,
    pub best_ask_price: Decimal,
    pub best_ask_amount: Decimal,
}

#[derive(Clone, Deserialize, Debug)]
pub struct ChannelParams {
    pub channel: String,
    pub data: TickerChannelData,
}

#[derive(Clone, Deserialize, Debug)]
pub struct DeribitPriceTick {
    pub jsonrpc: String,
    pub method: String,
    pub params: ChannelParams,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn deribit_price_tick() {
        let response_text =
            "{\"jsonrpc\":\"2.0\",\"method\":\"subscription\",\"params\":{\"channel\":\"quote.BTC-PERPETUAL\",\"data\":{\"timestamp\":1674442149364,\"instrument_name\":\"BTC-PERPETUAL\",\"best_bid_price\":22749.0,\"best_bid_amount\":71130.0,\"best_ask_price\":22749.5,\"best_ask_amount\":46250.0}}}";
        let details = serde_json::from_str::<DeribitPriceTick>(response_text).unwrap();
        dbg!(details.clone());
        assert_eq!(details.params.data.best_bid_price, dec!(22749.0));
    }
}
