use chrono::Duration;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::Deserialize;
use shared::{
    payload::{OkexBtcUsdSwapOrderBookPayload, OrderBookPayload, PriceRaw},
    pubsub::Envelope,
    time::TimeStamp,
};
use std::{collections::BTreeMap, sync::Arc};
use thiserror::Error;
use tokio::sync::RwLock;

use crate::PriceConverter;

#[derive(Debug, Error)]
pub enum SnapshotCacheError {
    #[error("PayloadConversion: conversion from OrderBookPayload failed")]
    PayloadConversion,
    #[error("OutdatedSnapshot: last update was at {0}")]
    OutdatedSnapshot(TimeStamp),
    #[error("No snapshot data available")]
    NoSnapshotAvailable,
}

#[derive(Debug, Clone)]
pub struct SnapshotCache {
    inner: Arc<RwLock<SnapshotInner>>,
}
impl SnapshotCache {
    pub fn new(stale_after: Duration) -> Self {
        Self {
            inner: Arc::new(RwLock::new(SnapshotInner::new(stale_after))),
        }
    }

    pub async fn apply_update(&self, snapshot: Envelope<OkexBtcUsdSwapOrderBookPayload>) {
        self.inner.write().await.update_snapshot(snapshot);
    }

    pub async fn latest_snapshot(&self) -> Result<MOBSnapshot, SnapshotCacheError> {
        let snap = self.inner.read().await.latest_snapshot()?;
        Ok(snap)
    }
}

#[derive(Debug, Clone)]
struct SnapshotInner {
    stale_after: Duration,
    snapshot: Option<MOBSnapshot>,
}
impl SnapshotInner {
    fn new(stale_after: Duration) -> Self {
        Self {
            stale_after,
            snapshot: None,
        }
    }

    fn update_snapshot(&mut self, snap: Envelope<OkexBtcUsdSwapOrderBookPayload>) {
        let payload = snap.payload.0;

        if let Some(snap) = self.snapshot.clone() {
            if snap.timestamp > payload.timestamp {
                return;
            }
        }

        if let Ok(snapshot) = MOBSnapshot::try_from(payload) {
            self.snapshot = Some(snapshot);
        }
    }

    fn latest_snapshot(&self) -> Result<MOBSnapshot, SnapshotCacheError> {
        if let Some(ref snap) = self.snapshot {
            if snap.timestamp.duration_since() > self.stale_after {
                return Err(SnapshotCacheError::OutdatedSnapshot(snap.timestamp));
            }

            return Ok(snap.clone());
        }

        Err(SnapshotCacheError::NoSnapshotAvailable)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub struct QuotePrice(Decimal);
impl From<PriceRaw> for QuotePrice {
    fn from(price: PriceRaw) -> Self {
        let price = price.into();

        QuotePrice(price)
    }
}
impl QuotePrice {
    pub fn inner(&self) -> Decimal {
        self.0
    }
}

///Market Order Book (MOB)
#[derive(Debug, Clone, Deserialize)]
pub struct MOBSnapshot {
    pub asks: BTreeMap<QuotePrice, Decimal>,
    pub bids: BTreeMap<QuotePrice, Decimal>,
    pub timestamp: TimeStamp,
}
impl From<OrderBookPayload> for MOBSnapshot {
    fn from(value: OrderBookPayload) -> Self {
        let mut asks = BTreeMap::new();
        for (raw_price, raw_qty) in value.asks {
            let price = QuotePrice::from(raw_price);
            let quantity: Decimal = raw_qty.into();

            asks.insert(price, quantity);
        }
        let mut bids = BTreeMap::new();
        for (raw_price, raw_qty) in value.bids {
            let price = QuotePrice::from(raw_price);
            let quantity: Decimal = raw_qty.into();

            bids.insert(price, quantity);
        }

        Self {
            asks,
            bids,
            timestamp: value.timestamp,
        }
    }
}

impl MOBSnapshot {
    pub fn sell_usd(&self) -> PriceConverter {
        PriceConverter::new(self.ask_price_of_one_sat())
    }

    pub fn buy_usd(&self) -> PriceConverter {
        PriceConverter::new(self.bid_price_of_one_sat())
    }

    pub fn mid_price_of_one_sat(&self) -> Decimal {
        (self.ask_price_of_one_sat() + self.bid_price_of_one_sat()) / dec!(2)
    }

    pub fn bid_price_of_one_sat(&self) -> Decimal {
        let bids = &self.bids;
        let acc_price_by_size = bids.iter().fold(dec!(0), |acc, (price, quantity)| {
            acc + (price.inner() * quantity)
        });
        let acc_size = bids
            .iter()
            .fold(dec!(0), |acc, (_, quantity)| acc + quantity);

        let weighted_average_price = acc_price_by_size / acc_size;

        weighted_average_price
    }

    pub fn ask_price_of_one_sat(&self) -> Decimal {
        let asks = &self.asks;
        let acc_price_by_size = asks.iter().fold(dec!(0), |acc, (price, quantity)| {
            acc + (price.inner() * quantity)
        });
        let acc_size = asks
            .iter()
            .fold(dec!(0), |acc, (_, quantity)| acc + quantity);

        let weighted_average_price = acc_price_by_size / acc_size;

        weighted_average_price
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use rust_decimal_macros::dec;
    use shared::payload::{ExchangeIdRaw, QuantityRaw};

    #[derive(Debug, Deserialize)]
    struct SnapshotFixture {
        payload: MOBSnapshot,
    }

    fn load_order_book(filename: &str) -> anyhow::Result<SnapshotFixture> {
        let contents = fs::read_to_string(format!("./tests/fixtures/order-book-{}.json", filename))
            .expect(&format!("Couldn't load fixture {}", filename));

        let res = serde_json::from_str::<SnapshotFixture>(&contents)?;
        Ok(res)
    }

    #[test]
    fn convert_payload_to_snapshot() {
        let mut bids = BTreeMap::new();
        bids.insert(PriceRaw::from(dec!(100)), QuantityRaw::from(dec!(10)));
        let mut asks = BTreeMap::new();
        asks.insert(PriceRaw::from(dec!(100)), QuantityRaw::from(dec!(10)));

        let payload = OrderBookPayload {
            asks,
            bids,
            timestamp: TimeStamp::now(),
            exchange: ExchangeIdRaw::from("okex".to_string()),
        };

        let lob_snapshot =
            MOBSnapshot::try_from(payload.clone()).expect("payload conversion to snapshot failed");

        assert_eq!(lob_snapshot.timestamp, payload.timestamp);
        assert_eq!(lob_snapshot.asks.len(), payload.asks.len());
    }

    #[test]
    fn weighted_prices_from_snapshot_fixture() -> anyhow::Result<()> {
        let latest_snapshot = load_order_book("payload")?.payload;
        let weighted_ask_price = latest_snapshot.ask_price_of_one_sat();
        let weighted_bid_price = latest_snapshot.bid_price_of_one_sat();

        assert_eq!(weighted_ask_price, dec!(0.0205761869636376921016098785));
        assert_eq!(weighted_bid_price, dec!(0.0202002976499512272808711405));

        Ok(())
    }

    #[test]
    fn mid_price() {
        let mut asks = BTreeMap::new();
        asks.insert(QuotePrice(dec!(10000)), dec!(10));
        asks.insert(QuotePrice(dec!(10000)), dec!(10));

        let mut bids = BTreeMap::new();
        bids.insert(QuotePrice(dec!(5000)), dec!(10));
        bids.insert(QuotePrice(dec!(5000)), dec!(10));

        let snapshot = MOBSnapshot {
            asks,
            bids,
            timestamp: TimeStamp::now(),
        };
        let mid_price = snapshot.mid_price_of_one_sat();

        assert_eq!(mid_price, dec!(7500));
    }

    #[test]
    fn weighted_prices_from_constructed_snapshot() -> anyhow::Result<()> {
        let mut asks = BTreeMap::new();
        asks.insert(QuotePrice(dec!(100)), dec!(10));
        asks.insert(QuotePrice(dec!(110)), dec!(11));
        asks.insert(QuotePrice(dec!(120)), dec!(12));
        asks.insert(QuotePrice(dec!(130)), dec!(13));

        let mut bids = BTreeMap::new();
        bids.insert(QuotePrice(dec!(100)), dec!(10));
        bids.insert(QuotePrice(dec!(110)), dec!(11));
        bids.insert(QuotePrice(dec!(120)), dec!(12));
        bids.insert(QuotePrice(dec!(130)), dec!(13));

        let snapshot = MOBSnapshot {
            asks,
            bids,
            timestamp: TimeStamp::now(),
        };
        let weighted_ask_price = snapshot.ask_price_of_one_sat();
        let weighted_bid_price = snapshot.bid_price_of_one_sat();

        assert_eq!(weighted_ask_price.round(), dec!(116));
        assert_eq!(weighted_bid_price.round(), dec!(116));

        Ok(())
    }
}
