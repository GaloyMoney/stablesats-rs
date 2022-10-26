use itertools::Itertools;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use shared::{payload::*, time::*};
use std::collections::BTreeMap;

use crate::{ChannelArgs, PriceFeedError};

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderBookAction {
    Snapshot,
    Update,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Clone, Serialize)]
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

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct OrderBookChannelData {
    pub asks: Vec<PriceQuantity>,
    pub bids: Vec<PriceQuantity>,
    pub ts: TimeStampMilliStr,
    pub checksum: i32,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct OkexOrderBook {
    pub arg: ChannelArgs,
    pub action: OrderBookAction,
    pub data: Vec<OrderBookChannelData>,
}

#[derive(Debug, PartialOrd, PartialEq, Eq, Ord, Clone, Deserialize)]
#[serde(transparent)]
pub struct OrderPrice(Decimal);
impl From<Decimal> for OrderPrice {
    fn from(d: Decimal) -> Self {
        Self(d)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct OrderBookIncrement {
    pub asks: BTreeMap<OrderPrice, Decimal>,
    pub bids: BTreeMap<OrderPrice, Decimal>,
    pub timestamp: TimeStamp,
    pub new_checksum: i32,
    pub action: OrderBookAction,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CompleteOrderBook {
    asks: BTreeMap<OrderPrice, Decimal>,
    bids: BTreeMap<OrderPrice, Decimal>,
    timestamp: TimeStamp,
    checksum: i32,
}
impl TryFrom<OrderBookIncrement> for CompleteOrderBook {
    type Error = PriceFeedError;
    fn try_from(book: OrderBookIncrement) -> Result<Self, Self::Error> {
        let result = CompleteOrderBook {
            asks: book.asks,
            bids: book.bids,
            timestamp: book.timestamp,
            checksum: book.new_checksum,
        };
        result.verify_checksum()?;
        Ok(result)
    }
}

impl CompleteOrderBook {
    fn verify_checksum(&self) -> Result<(), PriceFeedError> {
        let cs_res = self.calculate_checksum();
        if cs_res != self.checksum {
            return Err(PriceFeedError::CheckSumValidation);
        }
        Ok(())
    }

    fn try_merge(&self, mut increment: OrderBookIncrement) -> Result<Self, PriceFeedError> {
        let new_book = match increment.action {
            OrderBookAction::Snapshot => CompleteOrderBook::try_from(increment.clone())?,
            OrderBookAction::Update => {
                // 1. For same prices
                //      1.1 Delete depth info if size == 0
                //      1.2 Replace depth info if size differs

                let mut new_book = self.clone();
                let _asks_update = increment.asks.retain(|_x, y| y != &mut dec!(0));
                let _bids_update = increment.bids.retain(|_x, y| y != &mut dec!(0));

                for (ask_price, ask_qty) in increment.asks.iter_mut() {
                    if new_book.asks.contains_key(&ask_price) {
                        if let Some(val) = new_book.asks.get(&ask_price) {
                            if val != ask_qty {
                                new_book.asks.insert(ask_price.clone(), *ask_qty);
                            }
                        }
                    }
                }

                for (bid_price, bid_qty) in increment.bids.iter_mut() {
                    if new_book.bids.contains_key(&bid_price) {
                        if let Some(val) = new_book.bids.get(&bid_price) {
                            if val != bid_qty {
                                new_book.bids.insert(bid_price.clone(), *bid_qty);
                            }
                        }
                    }
                }

                // 2. For different prices
                //      2.1 sort asks ascendingly and insert asks
                //      2.2 sort bids descendingly and insert bids
                for (ask_price, ask_qty) in increment.asks {
                    if !new_book.asks.contains_key(&ask_price) {
                        new_book.asks.insert(ask_price, ask_qty);
                    }
                }

                for (bid_price, bid_qty) in increment.bids {
                    if !new_book.bids.contains_key(&bid_price) {
                        new_book.bids.insert(bid_price, bid_qty);
                    }
                }

                new_book.timestamp = increment.timestamp;
                new_book.checksum = increment.new_checksum;

                new_book
            }
        };

        new_book.verify_checksum()?;

        Ok(new_book)
    }

    fn calculate_checksum(&self) -> i32 {
        let asks_list = self
            .asks
            .iter()
            .enumerate()
            .filter(|(index, _)| index < &OKEX_CHECKSUM_LIMIT)
            .map(|(_, (price, qty))| format!("{}:{}", price.0, qty))
            .collect::<Vec<String>>();

        let bids_list = self
            .bids
            .iter()
            .rev()
            .enumerate()
            .filter(|(index, _)| index < &OKEX_CHECKSUM_LIMIT)
            .map(|(_, (price, qty))| format!("{}:{}", price.0, qty))
            .collect::<Vec<String>>();

        let crc =
            Itertools::intersperse(bids_list.into_iter().interleave(asks_list), ":".to_string())
                .collect::<String>();
        let checksum_val = crc32fast::hash(crc.as_bytes());
        let calc_cs = checksum_val as i32;

        calc_cs
    }
}

impl From<CompleteOrderBook> for OrderBookPayload {
    fn from(book: CompleteOrderBook) -> Self {
        let mut asks_map = BTreeMap::new();
        for (ask_price, ask_qty) in book.asks {
            let _ = asks_map.insert(PriceRaw::from(ask_price.0), QuantityRaw::from(ask_qty));
        }

        let mut bids_map = BTreeMap::new();
        for (bid_price, bid_qty) in book.bids {
            let _ = bids_map.insert(PriceRaw::from(bid_price.0), QuantityRaw::from(bid_qty));
        }

        Self {
            asks: asks_map,
            bids: bids_map,
            timestamp: book.timestamp,
            exchange: ExchangeIdRaw::from(OKEX_EXCHANGE_ID),
        }
    }
}

#[derive(Clone)]
pub struct OrderBookCache {
    current: CompleteOrderBook,
}

impl OrderBookCache {
    pub fn new(book: CompleteOrderBook) -> Self {
        Self { current: book }
    }

    pub fn update_order_book(&mut self, book: OrderBookIncrement) -> Result<(), PriceFeedError> {
        self.current = self.current.try_merge(book)?;
        Ok(())
    }

    pub fn latest(&self) -> CompleteOrderBook {
        self.current.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn load_order_book(filename: &str) -> anyhow::Result<OkexOrderBook> {
        let contents = fs::read_to_string(format!(
            "okex-price/src/order_book/fixtures/{}.json",
            filename
        ))
        .expect("Couldn't load fixtures");

        let res = serde_json::from_str::<OkexOrderBook>(&contents)?;
        Ok(res)
    }

    #[test]
    fn merge() -> anyhow::Result<()> {
        let snapshot = load_order_book("snapshot")?;
        let update_1 = load_order_book("update_1")?;
        let _update_2 = load_order_book("update_2")?;
        let _update_3 = load_order_book("update_3")?;

        let order_book_incr = OrderBookIncrement::try_from(snapshot)?;
        let mut cache = OrderBookCache::new(order_book_incr.try_into()?);

        let incr_1 = OrderBookIncrement::try_from(update_1)?;
        let _merge_1 = cache.update_order_book(incr_1.clone());

        assert_eq!(cache.latest().checksum, incr_1.new_checksum);

        Ok(())
    }
}
