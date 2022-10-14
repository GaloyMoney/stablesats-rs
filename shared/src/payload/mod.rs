mod constants;
mod primitives;

use std::collections::HashMap;

use lazy_static::*;
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

pub struct PriceMatch {
    pub snap: OrderBookRaw,
    pub incr: OrderBookRaw,
}

pub struct PriceMatches {
    pub asks: Vec<PriceMatch>,
    pub bids: Vec<PriceMatch>,
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
    pub asks: Vec<OrderBookRaw>,
    pub bids: Vec<OrderBookRaw>,
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
            Self::update_snapshot(update).await
        }
    }

    async fn update_snapshot(incr: Self) -> Self {
        let read_guard = SNAP_CACHE.read().await;
        let snapshot = read_guard.get("snapshot").expect("Empty snapshot");

        let price_matches = Self::same_price(snapshot, incr);

        // if price_matches.len() > 0 {
        //     if Self::size_is_zero(price_matches).len() > 0 {
        //         // Delete incr depth data
        //         todo!()
        //     }

        //     if Self::size_differs(price_matches) {
        //         // 1. Replace original with incr
        //         // 2. Validate checksum
        //         // 3. Update snap_cache
        //         // 4. return cached snapshot
        //         todo!()
        //     }
        // } else {
        //     // 1. price sort
        //     // 2. insert depth info
        //     // 3. validate checksum
        //     // 4. Update snap_cache
        //     // 5. return cached snapshot
        //     todo!()
        // }

        unimplemented!()
    }

    fn same_price(local_snapshot: &Self, update: Self) -> Vec<PriceMatch> {
        let asks = local_snapshot
            .asks
            .iter()
            .cmp_by(other, cmp)
        unimplemented!()
    }

    // fn size_is_zero(price_matches: Vec<String>) -> Vec<String> {
    //     unimplemented!()
    // }

    // fn size_differs(price_matches: Vec<String>) -> Vec<String> {
    //     unimplemented!()
    // }
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
        let mut asks = Vec::new();
        for pq in payload_data.clone() {
            asks.push(OrderBookRaw::from_pq(pq[0], pq[1]));
        }

        let mut bids = Vec::new();
        for pq in payload_data {
            bids.push(OrderBookRaw::from_pq(pq[0], pq[1]));
        }

        let payload = OrderBookPayload {
            asks,
            bids,
            timestamp: TimeStamp::now(),
            checksum: CheckSumRaw::from(-855196043_i64),
            action: action,
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

        let order_book_payload =
            generate_payload(full_snapshot_price_qty, OrderBookActionRaw::Snapshot);
        let _snapshot_merge_res =
            OkexBtcUsdSwapOrderBookPayload::merge(order_book_payload.clone()).await;

        let incremental_price_qty = vec![[dec!(8477.98), dec!(425)], [dec!(8477), dec!(0)]];

        // 2. Act
        let order_book_payload =
            generate_payload(incremental_price_qty, OrderBookActionRaw::Update);
        let update_merge_res =
            OkexBtcUsdSwapOrderBookPayload::merge(order_book_payload.clone()).await;

        // 3. Assert
        println!("Test");
        assert_eq!(update_merge_res.0.asks.len(), 5_usize);
    }
}
