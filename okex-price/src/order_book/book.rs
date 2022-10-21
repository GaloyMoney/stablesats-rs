use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::RwLock;

use crate::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::Deserialize;
use shared::time::{TimeStamp, TimeStampMilliStr};

// #[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
// #[serde(rename_all = "lowercase")]
// pub enum OrderBookAction {
//     Snapshot,
//     Update,
// }

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckSum(pub i32);

#[derive(Debug, PartialOrd, PartialEq, Eq, Ord, Clone)]
pub struct OrderPrice(pub Decimal);
impl std::ops::Deref for OrderPrice {
    type Target = Decimal;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderQuantity(pub Decimal);
impl std::ops::Deref for OrderQuantity {
    type Target = Decimal;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PriceQtyMap(pub BTreeMap<OrderPrice, OrderQuantity>);
impl std::ops::Deref for PriceQtyMap {
    type Target = BTreeMap<OrderPrice, OrderQuantity>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for PriceQtyMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone)]
pub struct PriceMatches {
    pub asks: PriceQtyMap,
    pub bids: PriceQtyMap,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentSnapshotInner {
    pub asks: PriceQtyMap,
    pub bids: PriceQtyMap,
    pub timestamp: TimeStamp,
    pub checksum: CheckSum,
    pub action: OrderBookAction,
}

#[derive(Debug, Clone)]
pub struct CurrentSnapshot {
    current: Arc<RwLock<CurrentSnapshotInner>>,
}

impl CurrentSnapshot {
    /// Create new order book snapshot with initial full load
    pub async fn new(snapshot: CurrentSnapshotInner) -> Self {
        Self::initial_snapshot(snapshot)
    }

    /// Update snapshot with incremental updates
    pub async fn merge(&self, update: CurrentSnapshotInner) -> Self {
        if update.action == OrderBookAction::Snapshot {
            Self::initial_snapshot(update)
        } else {
            let _update = self.current.write().await.update_snapshot(update);
            self.clone()
        }
    }

    /// Insert snapshot
    fn initial_snapshot(snapshot: CurrentSnapshotInner) -> Self {
        let current = Arc::new(RwLock::new(snapshot));
        Self { current }
    }

    /// Retrieve latest snapshot
    pub async fn latest_snapshot(&self) -> CurrentSnapshotInner {
        let guard = self.current.read().await;
        CurrentSnapshotInner {
            asks: guard.asks.clone(),
            bids: guard.bids.clone(),
            timestamp: guard.timestamp,
            checksum: guard.checksum.clone(),
            action: guard.action.clone(),
        }
    }
}

impl CurrentSnapshotInner {
    fn update_snapshot(&mut self, update: CurrentSnapshotInner) -> Self {
        let price_matches = self.same_price(&update);
        let (asks, bids) = (price_matches.clone().asks, price_matches.clone().bids);
        if !asks.is_empty() || !bids.is_empty() {
            let updated_price_matches = self.delete_empty_depth_data(price_matches);
            self.update_same_price(updated_price_matches);
            self.checksum = update.checksum;
            self.clone()
        } else {
            self.update_diff_price(&update);
            self.clone()
        }
    }

    fn same_price(&self, update: &Self) -> PriceMatches {
        let mut asks_map: BTreeMap<OrderPrice, OrderQuantity> = BTreeMap::new();
        for (price_u, qty_u) in update.asks.iter() {
            if self.asks.contains_key(price_u) {
                asks_map.insert(price_u.clone(), qty_u.clone());
            }
        }

        let mut bids_map: BTreeMap<OrderPrice, OrderQuantity> = BTreeMap::new();
        for (price_u, qty_u) in update.asks.iter() {
            if self.bids.contains_key(price_u) {
                bids_map.insert(price_u.clone(), qty_u.clone());
            }
        }

        PriceMatches {
            asks: PriceQtyMap(asks_map),
            bids: PriceQtyMap(bids_map),
        }
    }

    fn delete_empty_depth_data(&self, price_matches: PriceMatches) -> PriceMatches {
        let (mut asks, mut bids) = (price_matches.asks, price_matches.bids);
        asks.retain(|_x, y| *y != OrderQuantity(dec!(0)));
        bids.retain(|_x, y| *y != OrderQuantity(dec!(0)));

        PriceMatches { asks, bids }
    }

    fn update_same_price(&mut self, price_matches: PriceMatches) {
        let (asks, bids) = (price_matches.asks, price_matches.bids);
        asks.iter().for_each(|(ask_price, ask_qty)| {
            let snap_match = self
                .asks
                .get_mut(ask_price)
                .expect("Empty order book depth data ");

            if ask_qty != snap_match {
                *snap_match = ask_qty.clone();
            }
        });

        bids.iter().for_each(|(bid_price, bid_qty)| {
            let snap_match = self
                .bids
                .get_mut(bid_price)
                .expect("Empty order book depth data ");

            if bid_qty != snap_match {
                *snap_match = bid_qty.clone();
            }
        });
    }

    fn update_diff_price(&mut self, update: &Self) {
        let incr_asks = &update.asks;
        let incr_bids = &update.bids;

        incr_asks.iter().for_each(|(price, qty)| {
            self.asks.insert(price.clone(), qty.clone());
        });
        incr_bids.iter().for_each(|(price, qty)| {
            self.bids.insert(price.clone(), qty.clone());
        });

        self.checksum = update.checksum.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    fn generate_price_qty_map(list: Vec<[Decimal; 2]>) -> PriceQtyMap {
        let mut map = BTreeMap::new();
        for pq in list {
            map.insert(OrderPrice::from(pq[0]), OrderQuantity::from(pq[1]));
        }
        PriceQtyMap(map)
    }

    fn generate_snapshot(
        asks_data: Vec<[Decimal; 2]>,
        bids_data: Vec<[Decimal; 2]>,
        action: OrderBookAction,
        checksum: i32,
    ) -> CurrentSnapshotInner {
        let inner = CurrentSnapshotInner {
            asks: generate_price_qty_map(asks_data),
            bids: generate_price_qty_map(bids_data),
            timestamp: TimeStamp::now(),
            checksum: CheckSum::from(checksum),
            action,
        };

        inner
    }

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

    #[tokio::test]
    async fn cache_initial_full_load() {
        // 1. Arrange
        let asks_raw = vec![
            [dec!(8476.98), dec!(415)],
            [dec!(8477), dec!(7)],
            [dec!(8477.34), dec!(85)],
            [dec!(8477.56), dec!(1)],
            [dec!(8505.84), dec!(8)],
            [dec!(8506.37), dec!(85)],
            [dec!(8506.49), dec!(2)],
            [dec!(8506.96), dec!(100)],
        ];
        let bids_raw = vec![
            [dec!(8476.97), dec!(256)],
            [dec!(8475.55), dec!(101)],
            [dec!(8475.54), dec!(100)],
            [dec!(8475.3), dec!(1)],
            [dec!(8447.32), dec!(6)],
            [dec!(8447.02), dec!(246)],
            [dec!(8446.83), dec!(24)],
            [dec!(8446), dec!(95)],
        ];
        let snapshot_inner =
            generate_snapshot(asks_raw, bids_raw, OrderBookAction::Snapshot, -2102840145);

        // 2. Act
        let current_snapshot = CurrentSnapshot::new(snapshot_inner.clone()).await;
        let snapshot = current_snapshot.latest_snapshot().await;

        // 3. Assert
        assert_eq!(snapshot, snapshot_inner);
    }

    #[tokio::test]
    async fn incremental_update_same_price() {
        // 1. Arrange
        let asks_raw = vec![
            [dec!(8476.98), dec!(415)],
            [dec!(8477), dec!(7)],
            [dec!(8477.34), dec!(85)],
            [dec!(8477.56), dec!(1)],
            [dec!(8505.84), dec!(8)],
            [dec!(8506.37), dec!(85)],
            [dec!(8506.49), dec!(2)],
            [dec!(8506.96), dec!(100)],
        ];
        let bids_raw = vec![
            [dec!(8476.97), dec!(256)],
            [dec!(8475.55), dec!(101)],
            [dec!(8475.54), dec!(100)],
            [dec!(8475.3), dec!(1)],
            [dec!(8447.32), dec!(6)],
            [dec!(8447.02), dec!(246)],
            [dec!(8446.83), dec!(24)],
            [dec!(8446), dec!(95)],
        ];
        let initial_snapshot_inner =
            generate_snapshot(asks_raw, bids_raw, OrderBookAction::Snapshot, -2102840145);

        // 2. Act
        // 2.1 Initial full load snapshot
        let full_load = CurrentSnapshot::new(initial_snapshot_inner.clone()).await;

        let asks_raw = vec![[dec!(8476.98), dec!(415)], [dec!(8477), dec!(7)]];
        let bids_raw = vec![[dec!(8476.97), dec!(256)], [dec!(8475.55), dec!(101)]];

        // 2.2 Incremental update
        let update = generate_snapshot(asks_raw, bids_raw, OrderBookAction::Update, 2123921068);
        let _update_full_load = full_load.merge(update).await;
        let updated_full_load = full_load.latest_snapshot().await;

        // 3. Assert
        assert_eq!(updated_full_load.asks.len(), 8_usize);
        assert_eq!(updated_full_load.bids.len(), 8_usize);
        assert_eq!(initial_snapshot_inner.asks, updated_full_load.asks);
        assert_eq!(initial_snapshot_inner.bids, updated_full_load.bids);
    }

    #[tokio::test]
    async fn delete_empty_depth_data() {
        // 1. Arrange
        let asks_raw = vec![
            [dec!(8476.98), dec!(415)],
            [dec!(8477), dec!(7)],
            [dec!(8477.34), dec!(85)],
            [dec!(8477.56), dec!(1)],
            [dec!(8505.84), dec!(8)],
            [dec!(8506.37), dec!(85)],
            [dec!(8506.49), dec!(2)],
            [dec!(8506.96), dec!(100)],
        ];
        let bids_raw = vec![
            [dec!(8476.97), dec!(256)],
            [dec!(8475.55), dec!(101)],
            [dec!(8475.54), dec!(100)],
            [dec!(8475.3), dec!(1)],
            [dec!(8447.32), dec!(6)],
            [dec!(8447.02), dec!(246)],
            [dec!(8446.83), dec!(24)],
            [dec!(8446), dec!(95)],
        ];
        let initial_snapshot_inner =
            generate_snapshot(asks_raw, bids_raw, OrderBookAction::Snapshot, -2102840145);

        // 2. Act
        // 2.1 Initial full load snapshot
        let full_load = CurrentSnapshot::new(initial_snapshot_inner.clone()).await;

        let asks_raw = vec![[dec!(8476.98), dec!(415)], [dec!(8477), dec!(0)]];
        let bids_raw = vec![[dec!(8476.97), dec!(256)], [dec!(8475.55), dec!(0)]];

        // 2.2 Incremental update
        let update = generate_snapshot(asks_raw, bids_raw, OrderBookAction::Update, 925971892);

        // 2. Act
        let _update_full_load = full_load.merge(update.clone()).await;
        let update_full_load = full_load.latest_snapshot().await;

        // 3. Assert
        assert_eq!(initial_snapshot_inner.asks, update_full_load.asks);
        assert_eq!(initial_snapshot_inner.bids, update_full_load.bids);
    }

    #[tokio::test]
    async fn incremental_update_diff_prices() {
        // 1. Arrange
        let asks_raw = vec![
            [dec!(8476.98), dec!(415)],
            [dec!(8477), dec!(7)],
            [dec!(8477.34), dec!(85)],
            [dec!(8477.56), dec!(1)],
            [dec!(8505.84), dec!(8)],
            [dec!(8506.37), dec!(85)],
            [dec!(8506.49), dec!(2)],
            [dec!(8506.96), dec!(100)],
        ];
        let bids_raw = vec![
            [dec!(8476.97), dec!(256)],
            [dec!(8475.55), dec!(101)],
            [dec!(8475.54), dec!(100)],
            [dec!(8475.3), dec!(1)],
            [dec!(8447.32), dec!(6)],
            [dec!(8447.02), dec!(246)],
            [dec!(8446.83), dec!(24)],
            [dec!(8446), dec!(95)],
        ];
        let initial_snapshot_inner =
            generate_snapshot(asks_raw, bids_raw, OrderBookAction::Snapshot, -2102840145);

        // 2. Act
        // 2.1 Initial full load snapshot
        let full_load = CurrentSnapshot::new(initial_snapshot_inner.clone()).await;

        let asks_raw = vec![[dec!(8486.98), dec!(415)], [dec!(8487), dec!(7)]];
        let bids_raw = vec![[dec!(8486.97), dec!(256)], [dec!(8485.55), dec!(101)]];

        // 2.2 Incremental update
        let update = generate_snapshot(asks_raw, bids_raw, OrderBookAction::Update, 569993427);
        let _update_full_load = full_load.merge(update.clone()).await;
        let update_full_load = full_load.latest_snapshot().await;

        assert_eq!(initial_snapshot_inner.asks.len(), 8);
        assert_eq!(update_full_load.asks.len(), 10);
    }
}
