mod constants;
mod error;
mod primitives;

use std::collections::BTreeMap;
use std::collections::HashMap;

use crc32fast;
use itertools::Itertools;
use lazy_static::*;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use super::pubsub::*;
use super::time::*;

pub use constants::*;
pub use error::*;
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
    pub async fn merge(update: Self) -> Result<Self, PayloadError> {
        if update.action == OrderBookActionRaw::Snapshot {
            let initial_snapshot = Self::insert_snapshot(update).await?;
            Ok(initial_snapshot)
        } else {
            let mut write_guard = SNAP_CACHE.write().await;
            let local_full_load = write_guard
                .get_mut("snapshot")
                .expect("Empty local full snapshot");
            let update = local_full_load.update_snapshot(update).await?;
            Ok(update)
        }
    }

    async fn insert_snapshot(update: Self) -> Result<Self, PayloadError> {
        if !is_checksum_valid(&update) {
            return Err(PayloadError::CheckSumValidation);
        }

        // 1. Write access to update snapshot
        let mut write_guard = SNAP_CACHE.write().await;
        write_guard.insert("snapshot".to_string(), update.clone());
        drop(write_guard);
        // 2. Read access to return snapshot
        let read_guard = SNAP_CACHE.read().await;
        let snapshot = read_guard.get("snapshot").expect("Empty snapshot");
        Ok(snapshot.clone())
    }

    async fn update_snapshot(&mut self, incr: Self) -> Result<Self, PayloadError> {
        if !is_checksum_valid(&incr) {
            return Err(PayloadError::CheckSumValidation);
        }

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
            Ok(self.update_same_price(updated_price_matches))
        } else {
            // 1. [x] price sort (already sorted in BTreeMap)
            // 2. [x] insert depth info into snap_cache
            // 3. [ ] validate checksum
            // 4. [x ]return cached snapshot
            Ok(self.update_diff_price(&incr))
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
            if self.bids.contains_key(price_u) {
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

    fn update_same_price(&mut self, price_matches: PriceMatches) -> Self {
        let (asks, bids) = (price_matches.asks, price_matches.bids);
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

    fn update_diff_price(&mut self, incr: &Self) -> Self {
        let incr_asks = &incr.asks;
        let incr_bids = &incr.bids;

        incr_asks.iter().for_each(|(price, qty)| {
            self.asks.insert(*price, *qty);
        });
        incr_bids.iter().for_each(|(price, qty)| {
            self.bids.insert(*price, *qty);
        });

        self.clone()
    }
}

// Checks the validity of received order book data
fn is_checksum_valid(data: &OkexBtcUsdSwapOrderBookPayload) -> bool {
    // 1. [x] Get the key-value pairs from each BTreeMap entry, form a string separated by ':' and push string to vector
    // 2. [x] Do 1 above for asks and bids (in reverse)
    // 3. [x] Use itertools to interleave the vectors into a new collection (string)
    // 4. [x] CRC32 the resulting string
    let asks = &data.asks;
    let asks_list = asks
        .iter()
        .enumerate()
        .filter(|(idx, _pq)| idx <= &OKEX_CHECKSUM_LIMIT)
        .map(|(_idx, (price, qty))| format!("{}:{}", price, qty))
        .collect::<Vec<String>>();

    let bids = &data.bids;

    let crc_col = bids
        .iter()
        .rev()
        .enumerate()
        .filter(|(idx, _pq)| idx <= &OKEX_CHECKSUM_LIMIT)
        .map(|(_idx, (price, qty))| format!("{}:{}", price, qty))
        .interleave(asks_list);

    let crc = Itertools::intersperse(crc_col, ":".to_string()).collect::<String>();
    let recv_cs = data.checksum.clone();
    let cs = crc32fast::hash(crc.as_bytes());
    let calc_cs = CheckSumRaw::from(cs as i32); //crc32fast::hash(crc.as_bytes()));

    if calc_cs != recv_cs {
        return false;
    }

    true
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

    fn generate_order_book(list: Vec<[Decimal; 2]>) -> OrderBookRaw {
        let mut map = BTreeMap::new();
        for pq in list {
            map.insert(
                OrderBookPriceRaw::from(pq[0]),
                OrderBookQuantityRaw::from(pq[1]),
            );
        }
        OrderBookRaw(map)
    }

    fn generate_payload(
        asks_data: Vec<[Decimal; 2]>,
        bids_data: Vec<[Decimal; 2]>,
        action: OrderBookActionRaw,
        checksum: i32,
    ) -> OkexBtcUsdSwapOrderBookPayload {
        let payload = OrderBookPayload {
            asks: generate_order_book(asks_data),
            bids: generate_order_book(bids_data),
            timestamp: TimeStamp::now(),
            checksum: CheckSumRaw::from(checksum),
            action,
        };

        let order_book_payload = OkexBtcUsdSwapOrderBookPayload(payload);
        order_book_payload
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
        let order_book_payload = generate_payload(
            asks_raw,
            bids_raw,
            OrderBookActionRaw::Snapshot,
            -2102840145,
        );

        // 2. Act
        let merge_res = OkexBtcUsdSwapOrderBookPayload::merge(order_book_payload.clone())
            .await
            .expect("Expected order book payload");

        // 3. Assert
        assert_eq!(order_book_payload.action, merge_res.action);
        assert_eq!(order_book_payload.asks.len(), 8_usize);

        let read_guard = SNAP_CACHE.read().await;
        let snapshot_cache = read_guard
            .get("snapshot")
            .expect("No entry with key `snapshot`");
        assert_eq!(snapshot_cache.checksum, merge_res.checksum);
        assert_eq!(snapshot_cache.timestamp, merge_res.timestamp);
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
        let order_book_payload = generate_payload(
            asks_raw,
            bids_raw,
            OrderBookActionRaw::Snapshot,
            -2102840145,
        );

        // 2. Act
        // 2.1 Initial full load snapshot
        let _first_merge_res = OkexBtcUsdSwapOrderBookPayload::merge(order_book_payload.clone())
            .await
            .expect("Expected order book payload");

        let asks_raw = vec![[dec!(8476.98), dec!(415)], [dec!(8477), dec!(7)]];
        let bids_raw = vec![[dec!(8476.97), dec!(256)], [dec!(8475.55), dec!(101)]];

        // 2.2 Incremental update
        let order_book_payload =
            generate_payload(asks_raw, bids_raw, OrderBookActionRaw::Update, 2123921068);
        let update_merge_res = OkexBtcUsdSwapOrderBookPayload::merge(order_book_payload.clone())
            .await
            .expect("Expected order book payload");

        // 3. Assert
        assert_eq!(update_merge_res.asks.len(), 8_usize);
        assert_eq!(_first_merge_res.asks, update_merge_res.asks);
        assert_eq!(_first_merge_res.bids, update_merge_res.bids);
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
        let order_book_payload = generate_payload(
            asks_raw,
            bids_raw,
            OrderBookActionRaw::Snapshot,
            -2102840145,
        );

        // 2. Act
        // 2.1 Initial full load snapshot
        let first_merge_res = OkexBtcUsdSwapOrderBookPayload::merge(order_book_payload.clone())
            .await
            .expect("Expected order book payload");

        let asks_raw = vec![[dec!(8476.98), dec!(415)], [dec!(8477), dec!(0)]];
        let bids_raw = vec![[dec!(8476.97), dec!(256)], [dec!(8475.55), dec!(0)]];

        // 2.2 Incremental update
        let order_book_payload =
            generate_payload(asks_raw, bids_raw, OrderBookActionRaw::Update, 925971892);

        // 2. Act
        let update_merge_res = OkexBtcUsdSwapOrderBookPayload::merge(order_book_payload.clone())
            .await
            .expect("Expected order book payload");

        // 3. Assert
        assert_eq!(first_merge_res.asks, update_merge_res.asks);
        assert_eq!(first_merge_res.bids, update_merge_res.bids);
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
        let order_book_payload = generate_payload(
            asks_raw,
            bids_raw,
            OrderBookActionRaw::Snapshot,
            -2102840145,
        );

        // 2. Act
        // 2.1 Initial full load snapshot
        let first_merge_res = OkexBtcUsdSwapOrderBookPayload::merge(order_book_payload.clone())
            .await
            .expect("Expected order book payload");

        let asks_raw = vec![[dec!(8486.98), dec!(415)], [dec!(8487), dec!(7)]];
        let bids_raw = vec![[dec!(8486.97), dec!(256)], [dec!(8485.55), dec!(101)]];

        // 2.2 Incremental update
        let order_book_payload =
            generate_payload(asks_raw, bids_raw, OrderBookActionRaw::Update, 569993427);
        let update_merge_res = OkexBtcUsdSwapOrderBookPayload::merge(order_book_payload.clone())
            .await
            .expect("Expected order book payload");

        assert_eq!(first_merge_res.asks.len(), 8);
        assert_eq!(update_merge_res.asks.len(), 10);
    }

    #[test]
    fn depth_validation() {
        // 1. Arrange
        let asks_raw = vec![[dec!(8476.98), dec!(415)], [dec!(8477), dec!(7)]];
        let bids_raw = vec![
            [dec!(8476.97), dec!(256)],
            [dec!(8475.55), dec!(101)],
            [dec!(8475.54), dec!(100)],
        ];
        let order_book_payload = generate_payload(
            asks_raw,
            bids_raw,
            OrderBookActionRaw::Snapshot,
            -1404728636,
        );

        // 2. Act
        let is_valid = is_checksum_valid(&order_book_payload);

        // 3. Assert
        assert!(is_valid);
    }
}
