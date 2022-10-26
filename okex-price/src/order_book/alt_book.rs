use itertools::Itertools;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use shared::{payload::*, time::*};
use std::collections::BTreeMap;

use crate::{ChannelArgs, PriceFeedError};

const CHECKSUM_DEPTH_LIMIT: usize = 25;

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

    fn try_merge(&self, increment: OrderBookIncrement) -> Result<Self, PriceFeedError> {
        let new_book = match increment.action {
            OrderBookAction::Snapshot => CompleteOrderBook::try_from(increment.clone())?,
            OrderBookAction::Update => {
                let mut new_book = CompleteOrderBook {
                    timestamp: increment.timestamp,
                    checksum: increment.new_checksum,
                    ..self.clone()
                };

                for (ask_price, ask_qty) in increment.asks {
                    if ask_qty == Decimal::ZERO {
                        new_book.asks.remove(&ask_price);
                    } else {
                        new_book.asks.insert(ask_price, ask_qty);
                    }
                }

                for (bid_price, bid_qty) in increment.bids {
                    if bid_qty == Decimal::ZERO {
                        new_book.bids.remove(&bid_price);
                    } else {
                        new_book.bids.insert(bid_price, bid_qty);
                    }
                }

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
            .filter(|(index, _)| index < &CHECKSUM_DEPTH_LIMIT)
            .map(|(_, (price, qty))| format!("{}:{}", price.0, qty))
            .collect::<Vec<String>>();

        let bids_list = self
            .bids
            .iter()
            .rev()
            .enumerate()
            .take_while(|(index, _)| index < &CHECKSUM_DEPTH_LIMIT)
            .map(|(_, (price, qty))| format!("{}:{}", price.0, qty))
            .collect::<Vec<String>>();

        let crc =
            Itertools::intersperse(bids_list.into_iter().interleave(asks_list), ":".to_string())
                .collect::<String>();

        crc32fast::hash(crc.as_bytes()) as i32
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
        let contents = fs::read_to_string(format!("./src/order_book/fixtures/{}.json", filename))
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
        assert!(cache.update_order_book(incr_1.clone()).is_ok());
        assert_eq!(cache.latest().checksum, incr_1.new_checksum);

        Ok(())
    }
}
