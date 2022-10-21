use itertools::Itertools;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::Deserialize;
use shared::{payload::*, time::*};
use std::collections::BTreeMap;

use crate::{ChannelArgs, PriceFeedError};

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum OrderBookAction {
    Snapshot,
    Update,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
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
    pub checksum: i32,
}

#[derive(Debug, Deserialize)]
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

impl OrderBookIncrement {
    /// Delete entries with empty size/quantity
    fn delete_empty_qty(&mut self) {
        self.asks.retain(|a, _| a.0 != dec!(0));
        self.bids.retain(|c, _| c.0 != dec!(0));
    }
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
        Ok(())
    }

    fn try_merge(&self, increment: OrderBookIncrement) -> Result<Self, PriceFeedError> {
        let new_book = match increment.action {
            OrderBookAction::Snapshot => CompleteOrderBook::try_from(increment)?,
            OrderBookAction::Update => {
                let new_book = self.clone();
                new_book.execute_merge(increment)?
            }
        };
        // if !new_book.calculate_checksum() {
        //     // return Err(PriceFeedError::CheckSumValidation);
        //     println!("Checksum error");
        // }
        Ok(new_book)
    }

    fn execute_merge(&self, increment: OrderBookIncrement) -> Result<Self, PriceFeedError> {
        let mut price_matches = self.same_price(&increment);
        if !price_matches.asks.is_empty() || !price_matches.bids.is_empty() {
            price_matches.delete_empty_qty();
            let new_book = self.find_and_replace(&price_matches);
            Ok(new_book)
        } else {
            let new_book = self.insert(&increment);
            Ok(new_book)
        }
    }

    fn insert(&self, increment: &OrderBookIncrement) -> Self {
        let mut new_book = self.clone();

        for (price, qty) in increment.asks.clone() {
            let _ = new_book.asks.insert(price, qty);
        }

        for (price, qty) in increment.bids.clone() {
            let _ = new_book.bids.insert(price, qty);
        }

        new_book.checksum = increment.new_checksum;
        new_book.timestamp = increment.timestamp;

        Self {
            asks: new_book.asks.clone(),
            bids: new_book.bids.clone(),
            timestamp: new_book.timestamp,
            checksum: new_book.checksum,
        }
    }

    fn find_and_replace(&self, increment: &OrderBookIncrement) -> Self {
        let new_book = self.clone();

        let mut asks = new_book.asks;
        for (price, qty) in increment.asks.clone() {
            if let Some(val) = asks.get(&price) {
                if *val != qty {
                    let _ = asks.insert(price, qty);
                }
            }
        }

        let mut bids = new_book.bids;
        for (price, qty) in increment.bids.clone() {
            if let Some(val) = bids.get(&price) {
                if *val != qty {
                    let _ = bids.insert(price, qty);
                }
            }
        }

        CompleteOrderBook {
            asks,
            bids,
            timestamp: increment.timestamp,
            checksum: increment.new_checksum,
        }
    }

    fn calculate_checksum(&self) -> bool {
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
            .interleave(asks_list);

        let crc = Itertools::intersperse(bids_list, ":".to_string()).collect::<String>();
        let checksum_val = crc32fast::hash(crc.as_bytes());
        let calc_cs = checksum_val as i32;

        if calc_cs != self.checksum {
            return false;
        }

        true
    }

    fn same_price(&self, increment: &OrderBookIncrement) -> OrderBookIncrement {
        let mut asks = BTreeMap::new();
        for (incr_price, incr_qty) in increment.asks.iter() {
            for (complete_price, _) in self.asks.iter() {
                if incr_price == complete_price {
                    let _ = asks.insert(incr_price.clone(), *incr_qty);
                }
            }
        }

        let mut bids = BTreeMap::new();
        for (incr_price, incr_qty) in increment.bids.iter() {
            for (complete_price, _) in self.bids.iter() {
                if incr_price == complete_price {
                    let _ = bids.insert(incr_price.clone(), *incr_qty);
                }
            }
        }

        OrderBookIncrement {
            asks,
            bids,
            timestamp: increment.timestamp,
            new_checksum: increment.clone().new_checksum,
            action: increment.clone().action,
        }
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
    use rust_decimal_macros::dec;

    #[test]
    fn test_merge() {
        let mut map = BTreeMap::new();
        map.insert(OrderPrice(dec!(1900.10)), dec!(100));
        let full = OrderBookIncrement {
            asks: map.clone(),
            bids: map,
            timestamp: TimeStamp::now(),
            new_checksum: -123456789,
            action: OrderBookAction::Snapshot,
        };
        let order_book = CompleteOrderBook::try_from(full.clone()).expect("Complete order book");
        let mut cache = OrderBookCache::new(order_book);

        let mut map = BTreeMap::new();
        map.insert(OrderPrice(dec!(1900.20)), dec!(200));
        let incr = OrderBookIncrement {
            asks: map.clone(),
            bids: map,
            timestamp: TimeStamp::now(),
            new_checksum: 297630064,
            action: OrderBookAction::Update,
        };
        cache
            .update_order_book(incr.clone())
            .expect("Cache update failed");
        let latest = cache.latest();

        assert_eq!(latest.checksum, incr.new_checksum);
        assert_eq!(latest.asks.len(), 2_usize);
    }

    #[test]
    fn merge_same_price_same_qty() {
        let mut map = BTreeMap::new();
        map.insert(OrderPrice(dec!(1900.10)), dec!(100));
        let full = OrderBookIncrement {
            asks: map.clone(),
            bids: map,
            timestamp: TimeStamp::now(),
            new_checksum: -123456789,
            action: OrderBookAction::Snapshot,
        };

        let complete = CompleteOrderBook::try_from(full.clone()).expect("Complete order book");

        let mut map = BTreeMap::new();
        map.insert(OrderPrice(dec!(1900.10)), dec!(100));
        let incr = OrderBookIncrement {
            asks: map.clone(),
            bids: map,
            timestamp: TimeStamp::now(),
            new_checksum: 309356587,
            action: OrderBookAction::Update,
        };

        let merge_res = complete.try_merge(incr.clone()).expect("Merging failed");

        assert_eq!(merge_res.checksum, incr.new_checksum);
        assert_eq!(merge_res.asks.len(), 1_usize);
    }

    #[test]
    fn find_and_replace() {
        let mut map = BTreeMap::new();
        map.insert(OrderPrice(dec!(1900.10)), dec!(100));
        let full = OrderBookIncrement {
            asks: map.clone(),
            bids: map,
            timestamp: TimeStamp::now(),
            new_checksum: -123456789,
            action: OrderBookAction::Snapshot,
        };
        let complete = CompleteOrderBook::try_from(full.clone()).expect("Complete order book");
        let mut map = BTreeMap::new();
        map.insert(OrderPrice(dec!(1900.10)), dec!(110));
        let incr = OrderBookIncrement {
            asks: map.clone(),
            bids: map,
            timestamp: TimeStamp::now(),
            new_checksum: -987654321,
            action: OrderBookAction::Snapshot,
        };
        let new_book = complete.find_and_replace(&incr);

        assert_eq!(new_book.checksum, incr.new_checksum);
        assert_eq!(
            new_book
                .asks
                .get(&OrderPrice(dec!(1900.10)))
                .expect("Quantity"),
            incr.asks.get(&OrderPrice(dec!(1900.10))).expect("Quantity")
        );
    }

    #[test]
    fn insert() {
        let mut map = BTreeMap::new();
        map.insert(OrderPrice(dec!(1900.10)), dec!(100));
        let full = OrderBookIncrement {
            asks: map.clone(),
            bids: map,
            timestamp: TimeStamp::now(),
            new_checksum: -123456789,
            action: OrderBookAction::Snapshot,
        };
        let complete = CompleteOrderBook::try_from(full.clone()).expect("Complete order book");
        let mut map = BTreeMap::new();
        map.insert(OrderPrice(dec!(1900.20)), dec!(110));
        let incr = OrderBookIncrement {
            asks: map.clone(),
            bids: map,
            timestamp: TimeStamp::now(),
            new_checksum: -987654321,
            action: OrderBookAction::Snapshot,
        };
        let new_book = complete.insert(&incr);

        assert_eq!(new_book.asks.len(), 2_usize);
        assert_eq!(new_book.bids.len(), 2_usize);
        assert_eq!(new_book.checksum, incr.new_checksum);
        assert_eq!(
            new_book
                .asks
                .get(&OrderPrice(dec!(1900.10)))
                .expect("Quantity"),
            &dec!(100)
        );
        assert_eq!(
            new_book
                .asks
                .get(&OrderPrice(dec!(1900.20)))
                .expect("Quantity"),
            &dec!(110)
        );
    }
}
