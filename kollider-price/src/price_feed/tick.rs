use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Default, Debug, Clone, Deserialize)]
pub struct KolliderPriceTickerRoot {
    pub data: KolliderPriceTicker,
    #[serde(rename = "type")]
    pub type_str: String,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct KolliderPriceTicker {
    pub best_ask: Decimal,
    pub best_bid: Decimal,
    pub last_price: Decimal,
    pub last_quantity: i64,
    pub last_side: String,
    pub mid: String,
    pub symbol: String,
}
