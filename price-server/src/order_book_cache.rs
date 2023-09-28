use chrono::Duration;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::Deserialize;
use std::{collections::BTreeMap, sync::Arc};
use thiserror::Error;
use tokio::sync::RwLock;

use shared::{
    payload::{OrderBookPayload, PriceRaw, VolumeInCentsRaw},
    time::TimeStamp,
};

use crate::{
    currency::{UsdCents, VolumePicker},
    error::ExchangePriceCacheError,
    exchange_tick_cache,
    price_mixer::{PriceProvider, SidePicker},
    ExchangePriceCacheConfig, VolumeBasedPriceConverter,
};

#[derive(Debug, Error)]
pub enum OrderBookCacheError {
    #[error("PayloadConversion: conversion from OrderBookPayload failed")]
    PayloadConversion,
    #[error("OutdatedSnapshot: last update was at {0}")]
    OutdatedSnapshot(TimeStamp),
    #[error("No snapshot data available")]
    NoSnapshotAvailable,
    #[error("No price-quantity entry in asks or bids side")]
    EmptySide,
}

#[derive(Debug, Clone)]
pub struct OrderBookCache {
    inner: Arc<RwLock<SnapshotInner>>,
    config: ExchangePriceCacheConfig,
}

#[async_trait::async_trait]
impl PriceProvider for OrderBookCache {
    async fn latest(&self) -> Result<Box<dyn SidePicker>, ExchangePriceCacheError> {
        if let Some(mock_price) = self.config.dev_mock_price_btc_in_usd {
            return Ok(Box::new(exchange_tick_cache::mock_price_tick(mock_price)));
        }
        let order_book = self.latest_snapshot().await?;
        Ok(Box::new(order_book))
    }
}

impl OrderBookCache {
    pub fn new(config: ExchangePriceCacheConfig) -> Self {
        Self {
            inner: Arc::new(RwLock::new(SnapshotInner::new(config.stale_after))),
            config,
        }
    }

    pub async fn apply_update(&self, snapshot: OrderBookPayload) {
        self.inner.write().await.update_snapshot(snapshot);
    }

    pub async fn latest_snapshot(&self) -> Result<OrderBookView, OrderBookCacheError> {
        let snap = self.inner.read().await.current()?;
        Ok(snap)
    }
}

#[derive(Debug, Clone)]
struct SnapshotInner {
    stale_after: Duration,
    snapshot: Option<OrderBookView>,
}
impl SnapshotInner {
    fn new(stale_after: Duration) -> Self {
        Self {
            stale_after,
            snapshot: None,
        }
    }

    fn update_snapshot(&mut self, snap: OrderBookPayload) {
        let payload = snap;

        if let Some(ref snap) = self.snapshot {
            if snap.timestamp > payload.timestamp {
                return;
            }
        }

        let snapshot = OrderBookView::from(payload);
        self.snapshot = Some(snapshot);
    }

    fn current(&self) -> Result<OrderBookView, OrderBookCacheError> {
        if let Some(ref snap) = self.snapshot {
            if snap.timestamp.duration_since() > self.stale_after {
                return Err(OrderBookCacheError::OutdatedSnapshot(snap.timestamp));
            }

            return Ok(snap.clone());
        }

        Err(OrderBookCacheError::NoSnapshotAvailable)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub struct QuotePriceCentsForOneSat(Decimal);
impl From<PriceRaw> for QuotePriceCentsForOneSat {
    fn from(price: PriceRaw) -> Self {
        Self(Decimal::from(price))
    }
}
impl QuotePriceCentsForOneSat {
    pub fn inner(&self) -> Decimal {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub struct VolumeInCents(Decimal);
impl VolumeInCents {
    pub fn inner(&self) -> Decimal {
        self.0
    }
}
impl From<VolumeInCentsRaw> for VolumeInCents {
    fn from(volume: VolumeInCentsRaw) -> Self {
        Self(Decimal::from(volume))
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderBookView {
    pub asks: BTreeMap<QuotePriceCentsForOneSat, VolumeInCents>,
    pub bids: BTreeMap<QuotePriceCentsForOneSat, VolumeInCents>,
    pub timestamp: TimeStamp,
}
impl From<OrderBookPayload> for OrderBookView {
    fn from(value: OrderBookPayload) -> Self {
        let asks = value
            .asks
            .into_iter()
            .map(|(price, qty)| {
                (
                    QuotePriceCentsForOneSat::from(price),
                    VolumeInCents::from(qty),
                )
            })
            .collect::<BTreeMap<QuotePriceCentsForOneSat, VolumeInCents>>();

        let bids = value
            .bids
            .into_iter()
            .map(|(price, qty)| {
                (
                    QuotePriceCentsForOneSat::from(price),
                    VolumeInCents::from(qty),
                )
            })
            .collect::<BTreeMap<QuotePriceCentsForOneSat, VolumeInCents>>();

        Self {
            asks,
            bids,
            timestamp: value.timestamp,
        }
    }
}

impl SidePicker for OrderBookView {
    fn sell_usd(&self) -> Box<dyn VolumePicker + '_> {
        Box::new(VolumeBasedPriceConverter::new(self.asks.iter()))
    }

    fn buy_usd(&self) -> Box<dyn VolumePicker + '_> {
        Box::new(VolumeBasedPriceConverter::new(self.bids.iter().rev()))
    }

    fn mid_price_of_one_sat(&self) -> UsdCents {
        let best_ask = self
            .best_ask_price_of_one_sat()
            .expect("Failed to retrieve best ask price");
        let best_bid = self
            .best_bid_price_of_one_sat()
            .expect("Failed to retrieve best bid price");

        let mid_price = (best_ask + best_bid) / dec!(2);

        UsdCents::from_decimal(mid_price)
    }
}

impl OrderBookView {
    pub fn sell_usd(
        &self,
    ) -> VolumeBasedPriceConverter<
        std::collections::btree_map::Iter<QuotePriceCentsForOneSat, VolumeInCents>,
    > {
        VolumeBasedPriceConverter::new(self.asks.iter())
    }

    pub fn buy_usd(
        &self,
    ) -> VolumeBasedPriceConverter<
        std::iter::Rev<std::collections::btree_map::Iter<QuotePriceCentsForOneSat, VolumeInCents>>,
    > {
        VolumeBasedPriceConverter::new(self.bids.iter().rev())
    }

    pub fn mid_price_of_one_sat(&self) -> Result<Decimal, OrderBookCacheError> {
        let best_ask = self.best_ask_price_of_one_sat()?;
        let best_bid = self.best_bid_price_of_one_sat()?;

        Ok((best_ask + best_bid) / dec!(2))
    }

    fn best_bid_price_of_one_sat(&self) -> Result<Decimal, OrderBookCacheError> {
        let bids_length = self.bids.iter().next_back();

        let (best_price, _) = bids_length.ok_or(OrderBookCacheError::EmptySide)?;

        Ok(best_price.inner())
    }

    fn best_ask_price_of_one_sat(&self) -> Result<Decimal, OrderBookCacheError> {
        let ask_length = self.asks.iter().next();

        let (best_price, _) = ask_length.ok_or(OrderBookCacheError::EmptySide)?;

        Ok(best_price.inner())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use rust_decimal_macros::dec;
    use shared::payload::{ExchangeIdRaw, VolumeInCentsRaw};

    #[test]
    fn convert_payload_to_snapshot() {
        let mut bids = BTreeMap::new();
        bids.insert(PriceRaw::from(dec!(100)), VolumeInCentsRaw::from(dec!(10)));
        let mut asks = BTreeMap::new();
        asks.insert(PriceRaw::from(dec!(100)), VolumeInCentsRaw::from(dec!(10)));

        let payload = OrderBookPayload {
            asks,
            bids,
            timestamp: TimeStamp::now(),
            exchange: ExchangeIdRaw::from("okex".to_string()),
        };

        let lob_snapshot = OrderBookView::try_from(payload.clone())
            .expect("payload conversion to snapshot failed");

        assert_eq!(lob_snapshot.timestamp, payload.timestamp);
        assert_eq!(lob_snapshot.asks.len(), payload.asks.len());
    }

    #[test]
    fn mid_price() -> anyhow::Result<()> {
        let mut asks = BTreeMap::new();
        asks.insert(
            QuotePriceCentsForOneSat(dec!(10_000)),
            VolumeInCents(dec!(10)),
        );
        asks.insert(
            QuotePriceCentsForOneSat(dec!(25_000)),
            VolumeInCents(dec!(10)),
        );

        let mut bids = BTreeMap::new();
        bids.insert(
            QuotePriceCentsForOneSat(dec!(5_000)),
            VolumeInCents(dec!(10)),
        );
        bids.insert(
            QuotePriceCentsForOneSat(dec!(15_000)),
            VolumeInCents(dec!(10)),
        );

        let snapshot = OrderBookView {
            asks,
            bids,
            timestamp: TimeStamp::now(),
        };
        let mid_price = snapshot.mid_price_of_one_sat()?;

        assert_eq!(mid_price, dec!(12_500));

        Ok(())
    }
}
