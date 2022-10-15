mod constants;
mod primitives;

use std::collections::BTreeMap;
use std::collections::HashMap;

use lazy_static::*;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use super::pubsub::*;
use super::time::*;

pub use constants::*;
pub use primitives::*;

lazy_static! {
    /// Global mutable snapshot cache
    static ref SNAP_CACHE: RwLock<HashMap<String, OkexBtcUsdSwapOrderBookPayload>> =
        RwLock::new(HashMap::new());
}

#[derive(Debug, Clone)]
pub struct PriceMatches {
    pub asks: BTreeMap<OrderBookPriceRaw, OrderBookQuantityRaw>,
    pub bids: BTreeMap<OrderBookPriceRaw, OrderBookQuantityRaw>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceMessagePayload {
    pub timestamp: TimeStamp,
    pub exchange: ExchangeIdRaw,
    pub instrument_id: InstrumentIdRaw,
    pub ask_price: PriceRatioRaw,
    pub bid_price: PriceRatioRaw,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OkexBtcUsdSwapPricePayload(pub PriceMessagePayload);
impl From<OkexBtcUsdSwapPricePayload> for PriceMessagePayload {
    fn from(payload: OkexBtcUsdSwapPricePayload) -> Self {
        payload.0
    }
}
impl std::ops::Deref for OkexBtcUsdSwapPricePayload {
    type Target = PriceMessagePayload;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Payload of snapshot of an order book
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookPayload {
    pub asks: OrderBookRaw,
    pub bids: OrderBookRaw,
    pub timestamp: TimeStamp,
    pub checksum: CheckSumRaw,
    pub action: OrderBookActionRaw,
}

/// Message to transmit order book payload
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OkexBtcUsdSwapOrderBookPayload(pub OrderBookPayload);
impl From<OkexBtcUsdSwapOrderBookPayload> for OrderBookPayload {
    fn from(payload: OkexBtcUsdSwapOrderBookPayload) -> Self {
        payload.0
    }
}
impl std::ops::Deref for OkexBtcUsdSwapOrderBookPayload {
    type Target = OrderBookPayload;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for OkexBtcUsdSwapOrderBookPayload {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl OkexBtcUsdSwapOrderBookPayload {
    pub async fn merge(update: Self) -> Self {
        if update.0.action == OrderBookActionRaw::Snapshot {
            // 1. Write access to update snapshot
            let mut write_guard = SNAP_CACHE.write().await;
            write_guard.insert("snapshot".to_string(), update.clone());
            drop(write_guard);
            // 2. Read access to return snapshot
            let read_guard = SNAP_CACHE.read().await;
            let snapshot = read_guard.get("snapshot").expect("Empty snapshot");
            snapshot.clone()
        } else {
            let mut write_guard = SNAP_CACHE.write().await;
            let local_full_load = write_guard
                .get_mut("snapshot")
                .expect("Empty local full snapshot");
            local_full_load.update_snapshot(update).await
        }
    }

    async fn update_snapshot(&mut self, incr: Self) -> Self {
        // let read_guard = SNAP_CACHE.read().await;
        // let snapshot = read_guard.get("snapshot").expect("Empty snapshot");

        let price_matches = self.same_price(&incr);
        let (asks, bids) = (price_matches.clone().asks, price_matches.clone().bids);

        if !asks.is_empty() || !bids.is_empty() {
            // if price_matches.len() > 0 {
            //    1. [x] delete_empty_depth_data

            let updated_price_matches = self.delete_empty_depth_data(price_matches);

            // update_snapshot_with_incr if quantity differs {
            //   1. [x] Replace original with incr
            //   2. [ ] Validate checksum
            //   3. [x] Update snap_cache
            //   4. [x] return cached snapshot
            // }

            self.update_snapshot_with_incr(updated_price_matches).await
        } else {
            // 1. price sort
            // 2. insert depth info
            // 3. validate checksum
            // 4. Update snap_cache
            // 5. return cached snapshot
            todo!()
        }
    }

    fn same_price(&self, update: &Self) -> PriceMatches {
        let mut asks_map: BTreeMap<OrderBookPriceRaw, OrderBookQuantityRaw> = BTreeMap::new();
        for (price_u, qty_u) in update.asks.iter() {
            if self.asks.contains_key(price_u) {
                asks_map.insert(*price_u, *qty_u);
            }
        }

        let mut bids_map: BTreeMap<OrderBookPriceRaw, OrderBookQuantityRaw> = BTreeMap::new();
        for (price_u, qty_u) in update.asks.iter() {
            if self.asks.contains_key(price_u) {
                bids_map.insert(*price_u, *qty_u);
            }
        }

        PriceMatches {
            asks: asks_map,
            bids: bids_map,
        }
    }

    fn delete_empty_depth_data(&self, price_matches: PriceMatches) -> PriceMatches {
        let (mut asks, mut bids) = (price_matches.asks, price_matches.bids);
        asks.retain(|_x, y| *y != dec!(0));
        bids.retain(|_x, y| *y != dec!(0));

        PriceMatches { asks, bids }
    }

    async fn update_snapshot_with_incr(&mut self, price_matches: PriceMatches) -> Self {
        let (asks, bids) = (price_matches.asks, price_matches.bids);
        // let mut write_guard = SNAP_CACHE.write().await;
        // let snapshot = write_guard.get_mut("snapshot").expect("Empty snapshot");

        asks.iter().for_each(|(ask_price, ask_qty)| {
            let snap_match = self
                .asks
                .get_mut(ask_price)
                .expect("Empty order book depth data ");

            if ask_qty != snap_match {
                *snap_match = *ask_qty;
            }
        });

        bids.iter().for_each(|(bid_price, bid_qty)| {
            let snap_match = self
                .bids
                .get_mut(bid_price)
                .expect("Empty order book depth data ");

            if bid_qty != snap_match {
                *snap_match = *bid_qty;
            }
        });

        self.clone()
    }
}

crate::payload! { OkexBtcUsdSwapPricePayload, "price.okex.btc-usd-swap" }
crate::payload! { OkexBtcUsdSwapOrderBookPayload, "snapshot.okex.btc-usd-swap" }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthUsdLiabilityPayload {
    pub liability: SyntheticCentLiability,
}
crate::payload! { SynthUsdLiabilityPayload, "liability.synth-usd" }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OkexBtcUsdSwapPositionPayload {
    pub exchange: ExchangeIdRaw,
    pub instrument_id: InstrumentIdRaw,
    pub signed_usd_exposure: SyntheticCentExposure,
}
crate::payload! { OkexBtcUsdSwapPositionPayload, "position.okex.btc-usd-swap" }

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    fn generate_payload(
        payload_data: Vec<[Decimal; 2]>,
        action: OrderBookActionRaw,
    ) -> OkexBtcUsdSwapOrderBookPayload {
        let mut asks_map = BTreeMap::new();

        for pq in payload_data.clone() {
            asks_map.insert(
                OrderBookPriceRaw::from(pq[0]),
                OrderBookQuantityRaw::from(pq[1]),
            );
        }
        let asks = OrderBookRaw(asks_map);

        let mut bids_map = BTreeMap::new();

        for pq in payload_data.clone() {
            bids_map.insert(
                OrderBookPriceRaw::from(pq[0]),
                OrderBookQuantityRaw::from(pq[1]),
            );
        }
        let bids = OrderBookRaw(bids_map);

        let payload = OrderBookPayload {
            asks,
            bids,
            timestamp: TimeStamp::now(),
            checksum: CheckSumRaw::from(-855196043_i64),
            action,
        };

        let order_book_payload = OkexBtcUsdSwapOrderBookPayload(payload);
        order_book_payload
    }

    #[tokio::test]
    async fn local_full_load() {
        // 1. Arrange
        let price_qty = vec![
            [dec!(8476.98), dec!(415)],
            [dec!(8477), dec!(7)],
            [dec!(8477.34), dec!(85)],
            [dec!(8477.56), dec!(1)],
            [dec!(8505.84), dec!(8)],
        ];
        let order_book_payload = generate_payload(price_qty, OrderBookActionRaw::Snapshot);

        // 2. Act
        let merge_res = OkexBtcUsdSwapOrderBookPayload::merge(order_book_payload.clone()).await;

        // 3. Assert
        assert_eq!(order_book_payload.0.action, merge_res.0.action);
        assert_eq!(order_book_payload.0.asks.len(), 5_usize);

        let read_guard = SNAP_CACHE.read().await;
        let snapshot_cache = read_guard.get("snapshot").unwrap();
        assert_eq!(snapshot_cache.0.checksum, merge_res.0.checksum);
        assert_eq!(snapshot_cache.0.timestamp, merge_res.0.timestamp);
    }

    #[tokio::test]
    async fn incremental_update() {
        // 1. Arrange
        let full_snapshot_price_qty = vec![
            [dec!(8476.98), dec!(415)],
            [dec!(8477), dec!(7)],
            [dec!(8477.34), dec!(85)],
            [dec!(8477.56), dec!(1)],
            [dec!(8505.84), dec!(8)],
        ];

        // 2. Act
        // 2.1 Initial full load snapshot
        let order_book_payload =
            generate_payload(full_snapshot_price_qty, OrderBookActionRaw::Snapshot);
        let _snapshot_merge_res =
            OkexBtcUsdSwapOrderBookPayload::merge(order_book_payload.clone()).await;

        // 2.2 Incremental update
        let incremental_price_qty = vec![[dec!(8477.98), dec!(425)], [dec!(8477), dec!(0)]];
        let order_book_payload =
            generate_payload(incremental_price_qty, OrderBookActionRaw::Update);
        let update_merge_res =
            OkexBtcUsdSwapOrderBookPayload::merge(order_book_payload.clone()).await;

        // 3. Assert
        assert_eq!(update_merge_res.0.asks.len(), 5_usize);
    }

    #[test]
    fn group_by_same_price() {
        // 1. Arrange
        let full_snapshot_price_qty = vec![
            [dec!(8476.98), dec!(415)],
            [dec!(8477), dec!(7)],
            [dec!(8477.34), dec!(85)],
            [dec!(8477.56), dec!(1)],
            [dec!(8505.84), dec!(8)],
        ];
        let incremental_price_qty = vec![[dec!(8477.98), dec!(425)], [dec!(8477), dec!(0)]];

        let initial_snapshot =
            generate_payload(full_snapshot_price_qty, OrderBookActionRaw::Snapshot);
        let update = generate_payload(incremental_price_qty, OrderBookActionRaw::Update);

        // 2. Act
        let res = OkexBtcUsdSwapOrderBookPayload::same_price(&initial_snapshot, &update);

        // 3. Assert
        assert_eq!(res.asks.len(), 1);
        assert_eq!(res.bids.len(), 1);
    }

    #[test]
    fn delete_empty_depth_data() {
        // 1. Arrange
        let full_snapshot_price_qty = vec![
            [dec!(8476.98), dec!(415)],
            [dec!(8477), dec!(7)],
            [dec!(8477.34), dec!(85)],
            [dec!(8477.56), dec!(1)],
            [dec!(8505.84), dec!(8)],
        ];
        let incremental_price_qty = vec![[dec!(8477.98), dec!(425)], [dec!(8477), dec!(0)]];

        let initial_snapshot =
            generate_payload(full_snapshot_price_qty, OrderBookActionRaw::Snapshot);
        let update = generate_payload(incremental_price_qty, OrderBookActionRaw::Update);

        // 2. Act
        let price_matches = initial_snapshot.same_price(&update);
        let updated_price_matches = initial_snapshot.delete_empty_depth_data(price_matches);

        // 3. Assert
        assert_eq!(updated_price_matches.asks.len(), 0);
        assert_eq!(updated_price_matches.bids.len(), 0);
    }

    #[tokio::test]
    async fn update_snapshot_with_incr() {
        // 1. Arrange
        let full_snapshot_price_qty = vec![
            [dec!(8476.98), dec!(415)],
            [dec!(8477), dec!(7)],
            [dec!(8477.34), dec!(85)],
            [dec!(8477.56), dec!(1)],
            [dec!(8505.84), dec!(8)],
        ];
        let incremental_price_qty = vec![[dec!(8476.98), dec!(425)], [dec!(8477), dec!(8)]];

        let initial_snapshot =
            generate_payload(full_snapshot_price_qty, OrderBookActionRaw::Snapshot);
        let update = generate_payload(incremental_price_qty, OrderBookActionRaw::Update);

        // 2. Act
        let initial_full_load = OkexBtcUsdSwapOrderBookPayload::merge(initial_snapshot).await;
        let updated_full_load = OkexBtcUsdSwapOrderBookPayload::merge(update).await;

        // 3. Assert
        assert_eq!(
            initial_full_load
                .asks
                .get(&OrderBookPriceRaw::from(dec!(8476.98)))
                .expect("No matching price"),
            &OrderBookQuantityRaw::from(dec!(415))
        );
        assert_eq!(
            updated_full_load
                .asks
                .get(&OrderBookPriceRaw::from(dec!(8476.98)))
                .expect("No matching price"),
            &OrderBookQuantityRaw::from(dec!(425))
        );
        assert_eq!(
            initial_full_load
                .asks
                .get(&OrderBookPriceRaw::from(dec!(8477)))
                .expect("No matching price"),
            &OrderBookQuantityRaw::from(dec!(7))
        );
        assert_eq!(
            updated_full_load
                .asks
                .get(&OrderBookPriceRaw::from(dec!(8477)))
                .expect("No matching price"),
            &OrderBookQuantityRaw::from(dec!(8))
        );
    }
}
