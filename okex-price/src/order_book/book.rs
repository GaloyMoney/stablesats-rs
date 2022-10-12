use crate::okex_shared::ChannelArgs;
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
#[serde(from = "PriceQuantityRaw")]
pub struct PriceQuantity {
    pub price: Decimal,
    pub quantity: Decimal,
}

#[derive(Debug, Deserialize)]
#[serde(transparent)]
struct PriceQuantityRaw(Vec<Decimal>);

impl From<PriceQuantityRaw> for PriceQuantity {
    fn from(raw: PriceQuantityRaw) -> Self {
        let mut iter = raw.0.into_iter();
        let price = iter
            .next()
            .expect("Missing price element of order book price array");
        let quantity = iter
            .next()
            .expect("Missing quantity element of order book price array");
        Self { price, quantity }
    }
}

#[derive(Debug, Deserialize)]
pub struct OrderBookChannelData {
    pub asks: Vec<PriceQuantity>,
    pub bids: Vec<PriceQuantity>,
    pub ts: TimeStampMilliStr,
    pub checksum: i64,
}

#[derive(Debug, Deserialize)]
pub struct OkexOrderBook {
    pub arg: ChannelArgs,
    pub action: OrderBookAction,
    pub data: Vec<OrderBookChannelData>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_pricequantityraw() {
        let raw_data = r#"
                ["8476.98", "415", "0", "13"]
            "#;

        let price_qty_raw = serde_json::from_str::<PriceQuantityRaw>(raw_data)
            .expect("Failed to serialize to PriceQuantityRaw");

        let price_qty = PriceQuantity::from(price_qty_raw);
        assert_eq!(price_qty.price.to_string(), "8476.98".to_string());
        assert_eq!(price_qty.quantity.to_string(), "415".to_string());
    }
}
