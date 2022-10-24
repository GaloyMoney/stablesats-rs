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

#[derive(Debug, Deserialize, Eq, PartialEq, Clone)]
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

#[derive(Debug, Deserialize, Clone)]
pub struct OrderBookChannelData {
    pub asks: Vec<PriceQuantity>,
    pub bids: Vec<PriceQuantity>,
    pub ts: TimeStampMilliStr,
    pub checksum: i32,
}

#[derive(Debug, Deserialize, Clone)]
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

pub struct IncrementPrices {
    matches: OrderBookIncrement,
    diffs: OrderBookIncrement,
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
    fn delete_empty_qty(&self) -> Self {
        let mut incr = self.clone();
        incr.asks.retain(|_a, qty| qty != &mut dec!(0));
        incr.bids.retain(|_c, qty| qty != &mut dec!(0));

        incr
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
        let cs_res = self.calculate_checksum();
        if !cs_res {
            return Err(PriceFeedError::CheckSumValidation);
        }
        Ok(())
    }

    fn try_merge(&self, increment: OrderBookIncrement) -> Result<Self, PriceFeedError> {
        let new_book = match increment.action {
            OrderBookAction::Snapshot => CompleteOrderBook::try_from(increment)?,
            OrderBookAction::Update => {
                let new_book = self.clone();
                new_book.execute_merge(increment)
            }
        };

        if !new_book.calculate_checksum() {
            // 1. Re-subscribe to order book channel
            return Err(PriceFeedError::CheckSumValidation);
        }

        Ok(new_book)
    }

    fn execute_merge(&self, increment: OrderBookIncrement) -> Self {
        let mut incr_prices = self.categorize_by_price(&increment);
        let new_book = self.merge_to_order_book(&mut incr_prices.matches, &incr_prices.diffs);

        new_book
    }

    fn categorize_by_price(&self, incr: &OrderBookIncrement) -> IncrementPrices {
        // 1. Find matching price
        let matches = self.same_price(incr);
        let diffs = self.diff_price(incr);

        IncrementPrices { matches, diffs }
    }

    fn merge_to_order_book(
        &self,
        matching_incr: &mut OrderBookIncrement,
        diff_incr: &OrderBookIncrement,
    ) -> Self {
        let book = self.clone();
        let new_book = book.replace(matching_incr);
        let new_book = new_book.insert(diff_incr);

        new_book
    }

    fn replace(&self, matches: &mut OrderBookIncrement) -> Self {
        let non_empty_depth = matches.delete_empty_qty();
        let book = self.clone();
        let new_book = book.find_and_replace(&non_empty_depth);

        new_book
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
        let mut new_book = self.clone();

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

        new_book.timestamp = increment.timestamp;
        new_book.checksum = increment.new_checksum;

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
            .collect::<Vec<String>>();

        let crc =
            Itertools::intersperse(bids_list.into_iter().interleave(asks_list), ":".to_string())
                .collect::<String>();
        let checksum_val = crc32fast::hash(crc.as_bytes());
        let calc_cs = checksum_val as i32;

        if calc_cs != self.checksum {
            return false;
        }

        true
    }

    fn diff_price(&self, increment: &OrderBookIncrement) -> OrderBookIncrement {
        let mut asks = BTreeMap::new();
        for (incr_price, incr_qty) in increment.asks.iter() {
            if !self.asks.contains_key(incr_price) {
                let _ = asks.insert(incr_price.clone(), *incr_qty);
            }
        }

        let mut bids = BTreeMap::new();
        for (incr_price, incr_qty) in increment.bids.iter() {
            if !self.bids.contains_key(incr_price) {
                let _ = bids.insert(incr_price.clone(), *incr_qty);
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

    fn same_price(&self, increment: &OrderBookIncrement) -> OrderBookIncrement {
        let mut asks = BTreeMap::new();
        for (incr_price, incr_qty) in increment.asks.iter() {
            if self.asks.contains_key(incr_price) {
                let _ = asks.insert(incr_price.clone(), *incr_qty);
            }
        }

        let mut bids = BTreeMap::new();
        for (incr_price, incr_qty) in increment.bids.iter() {
            if self.bids.contains_key(incr_price) {
                let _ = bids.insert(incr_price.clone(), *incr_qty);
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

    fn order_book_cache() -> OrderBookCache {
        let mut ask_map = BTreeMap::new();
        ask_map.insert(OrderPrice(dec!(8476.98)), dec!(415));
        ask_map.insert(OrderPrice(dec!(8477)), dec!(7));
        ask_map.insert(OrderPrice(dec!(8477.34)), dec!(85));
        ask_map.insert(OrderPrice(dec!(8477.56)), dec!(1));
        ask_map.insert(OrderPrice(dec!(8505.84)), dec!(8));
        ask_map.insert(OrderPrice(dec!(8506.37)), dec!(85));
        ask_map.insert(OrderPrice(dec!(8506.49)), dec!(2));
        ask_map.insert(OrderPrice(dec!(8506.96)), dec!(100));

        let mut bid_map = BTreeMap::new();
        bid_map.insert(OrderPrice(dec!(8476.97)), dec!(256));
        bid_map.insert(OrderPrice(dec!(8475.55)), dec!(101));
        bid_map.insert(OrderPrice(dec!(8475.54)), dec!(100));
        bid_map.insert(OrderPrice(dec!(8475.3)), dec!(1));
        bid_map.insert(OrderPrice(dec!(8447.32)), dec!(6));
        bid_map.insert(OrderPrice(dec!(8447.02)), dec!(246));
        bid_map.insert(OrderPrice(dec!(8446.83)), dec!(24));
        bid_map.insert(OrderPrice(dec!(8446)), dec!(95));

        let full = OrderBookIncrement {
            asks: ask_map,
            bids: bid_map,
            timestamp: TimeStamp::now(),
            new_checksum: -2102840145,
            action: OrderBookAction::Snapshot,
        };
        let order_book = CompleteOrderBook::try_from(full.clone()).expect("Complete order book");
        let cache = OrderBookCache::new(order_book);
        cache
    }

    #[test]
    fn initial_full_load() {
        let cache = order_book_cache();

        assert_eq!(cache.latest().asks.len(), 8);
        assert_eq!(cache.latest().bids.len(), 8);
    }

    #[test]
    fn update_same_price_same_qty() {
        let mut cache = order_book_cache();

        let mut ask_map = BTreeMap::new();
        ask_map.insert(OrderPrice(dec!(8476.98)), dec!(415));
        ask_map.insert(OrderPrice(dec!(8477)), dec!(7));

        let mut bid_map = BTreeMap::new();
        bid_map.insert(OrderPrice(dec!(8476.97)), dec!(256));
        bid_map.insert(OrderPrice(dec!(8475.55)), dec!(101));
        bid_map.insert(OrderPrice(dec!(8475.54)), dec!(100));

        let incr = OrderBookIncrement {
            asks: ask_map,
            bids: bid_map,
            timestamp: TimeStamp::now(),
            new_checksum: -2102840145,
            action: OrderBookAction::Update,
        };

        let _update = cache.update_order_book(incr);

        assert_eq!(cache.latest().asks.len(), 8);
        assert_eq!(cache.latest().bids.len(), 8);
    }

    #[test]
    fn update_same_price_diff_qty() {
        let mut cache = order_book_cache();

        let mut ask_map = BTreeMap::new();
        ask_map.insert(OrderPrice(dec!(8476.98)), dec!(416));
        ask_map.insert(OrderPrice(dec!(8477)), dec!(8));

        let mut bid_map = BTreeMap::new();
        bid_map.insert(OrderPrice(dec!(8476.97)), dec!(257));
        bid_map.insert(OrderPrice(dec!(8475.55)), dec!(102));
        bid_map.insert(OrderPrice(dec!(8475.54)), dec!(101));

        let incr = OrderBookIncrement {
            asks: ask_map,
            bids: bid_map,
            timestamp: TimeStamp::now(),
            new_checksum: 1213749428,
            action: OrderBookAction::Update,
        };

        let _update = cache.update_order_book(incr);

        assert_eq!(cache.latest().asks.len(), 8);
        assert_eq!(cache.latest().bids.len(), 8);
        assert_eq!(
            cache.latest().asks.get(&OrderPrice(dec!(8476.98))).unwrap(),
            &dec!(416)
        );
    }

    #[test]
    fn update_diff_prices() {
        let mut cache = order_book_cache();

        let mut ask_map = BTreeMap::new();
        ask_map.insert(OrderPrice(dec!(8477.98)), dec!(416));
        ask_map.insert(OrderPrice(dec!(8478)), dec!(8));

        let mut bid_map = BTreeMap::new();
        bid_map.insert(OrderPrice(dec!(8477.97)), dec!(257));
        bid_map.insert(OrderPrice(dec!(8476.55)), dec!(102));
        bid_map.insert(OrderPrice(dec!(8476.54)), dec!(101));

        let incr = OrderBookIncrement {
            asks: ask_map,
            bids: bid_map,
            timestamp: TimeStamp::now(),
            new_checksum: 978115032,
            action: OrderBookAction::Update,
        };

        let _update = cache.update_order_book(incr);

        assert_eq!(cache.latest().asks.len(), 10);
        assert_eq!(cache.latest().bids.len(), 11);
    }
}
