use rust_decimal::Decimal;
use serde::Deserialize;
use shared::time::TimeStamp;
use std::collections::BTreeMap;

use crate::PriceFeedError;

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum OrderBookAction {
    Snapshot,
    Update,
}

#[derive(Debug, PartialOrd, PartialEq, Eq, Ord, Clone, Deserialize)]
#[serde(transparent)]
pub struct OrderPrice(Decimal);

#[derive(Debug, Deserialize)]
pub struct OrderBookIncrement {
    pub asks: BTreeMap<OrderPrice, Decimal>,
    pub bids: BTreeMap<OrderPrice, Decimal>,
    pub timestamp: TimeStamp,
    pub new_checksum: i32,
    pub action: OrderBookAction,
}

#[derive(Clone)]
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
    fn calculate_checksum() {}

    fn try_merge(&self, increment: OrderBookIncrement) -> Result<Self, PriceFeedError> {
        let new_book = match increment.action {
            OrderBookAction::Snapshot => CompleteOrderBook::try_from(increment)?,
            OrderBookAction::Update => {
                let new_book = self.clone();
                // execute merge
                new_book
            }
        };
        // calculate checksum
        Ok(new_book)
    }
}

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
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_merge() {}
}
