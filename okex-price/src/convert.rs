use std::collections::BTreeMap;

use crate::*;
use itertools::Itertools;
use rust_decimal::Decimal;
use shared::{payload::*, time::*};

use super::price_tick::{OkexPriceTick, PriceFeedError};

impl TryFrom<OkexPriceTick> for OkexBtcUsdSwapPricePayload {
    type Error = PriceFeedError;

    fn try_from(OkexPriceTick { arg, data }: OkexPriceTick) -> Result<Self, Self::Error> {
        let first_tick = data.first().ok_or(PriceFeedError::EmptyPriceData)?;

        Ok(OkexBtcUsdSwapPricePayload(PriceMessagePayload {
            exchange: ExchangeIdRaw::from(OKEX_EXCHANGE_ID),
            instrument_id: InstrumentIdRaw::from(arg.inst_id),
            timestamp: TimeStamp::try_from(&first_tick.ts)?,
            ask_price: PriceRatioRaw::from_one_btc_in_usd_price(first_tick.ask_px),
            bid_price: PriceRatioRaw::from_one_btc_in_usd_price(first_tick.bid_px),
        }))
    }
}

impl From<OrderBookAction> for OrderBookActionRaw {
    fn from(action: OrderBookAction) -> Self {
        match action {
            OrderBookAction::Snapshot => Self::Snapshot,
            OrderBookAction::Update => Self::Update,
        }
    }
}

impl TryFrom<OkexOrderBook> for CurrentSnapshotInner {
    type Error = PriceFeedError;

    fn try_from(value: OkexOrderBook) -> Result<Self, Self::Error> {
        let snapshot = value
            .data
            .first()
            .ok_or(PriceFeedError::EmptyOrderBookData)?;
        let action = value.action;

        let (asks, bids, ts, checksum) = (
            &snapshot.asks,
            &snapshot.bids,
            &snapshot.ts,
            &snapshot.checksum,
        );

        if !is_checksum_valid(asks, bids, *checksum) {
            return Err(PriceFeedError::CheckSumValidation);
        }

        Ok(CurrentSnapshotInner {
            asks: PriceQtyMap::from(asks),
            bids: PriceQtyMap::from(bids),
            timestamp: TimeStamp::try_from(ts)?,
            checksum: CheckSum::from(*checksum),
            action,
        })
    }
}

impl From<&Vec<PriceQuantity>> for PriceQtyMap {
    fn from(list: &Vec<PriceQuantity>) -> Self {
        let mut map = BTreeMap::new();
        for val in list {
            map.insert(
                OrderPrice::from(val.price),
                OrderQuantity::from(val.quantity),
            );
        }

        Self(map)
    }
}

impl From<i32> for CheckSum {
    fn from(val: i32) -> Self {
        Self(val)
    }
}

impl From<Decimal> for OrderPrice {
    fn from(d: Decimal) -> Self {
        Self(d)
    }
}

impl From<Decimal> for OrderQuantity {
    fn from(d: Decimal) -> Self {
        Self(d)
    }
}

impl From<CurrentSnapshotInner> for OkexBtcUsdSwapOrderBookPayload {
    fn from(snap: CurrentSnapshotInner) -> Self {
        Self(OrderBookPayload {
            asks: PriceQtyMapRaw::from(snap.asks),
            bids: PriceQtyMapRaw::from(snap.bids),
            timestamp: snap.timestamp,
            exchange: ExchangeIdRaw::from(OKEX_EXCHANGE_ID),
        })
    }
}

impl From<PriceQtyMap> for PriceQtyMapRaw {
    fn from(map: PriceQtyMap) -> Self {
        let mut raw_map = BTreeMap::new();

        for (price, qty) in map.iter() {
            raw_map.insert(PriceRaw::from(price.0), QuantityRaw::from(qty.0));
        }

        Self(raw_map)
    }
}

/// Checks the validity of received order book data
fn is_checksum_valid(asks: &[PriceQuantity], bids: &[PriceQuantity], checksum: i32) -> bool {
    // 0. [x] Convert vector of asks and bids to BTreeMap
    // 1. [x] Get the key-value pairs from each BTreeMap entry, form a string separated by ':' and push string to vector
    // 2. [x] Do 1 above for asks and bids (in reverse)
    // 3. [x] Use itertools to interleave the vectors into a new collection (string)
    // 4. [x] CRC32 the resulting string
    let mut asks_map = BTreeMap::new();
    for entry in asks {
        asks_map.insert(entry.price, entry.quantity);
    }
    let mut bids_map = BTreeMap::new();
    for entry in bids {
        bids_map.insert(entry.price, entry.quantity);
    }

    let asks_list = asks_map
        .iter()
        .enumerate()
        .filter(|(idx, _pq)| *idx <= OKEX_CHECKSUM_LIMIT)
        .map(|(_idx, val)| format!("{}:{}", val.0, val.1))
        .collect::<Vec<String>>();

    let crc_col = bids_map
        .iter()
        .rev()
        .enumerate()
        .filter(|(idx, _pq)| idx <= &OKEX_CHECKSUM_LIMIT)
        .map(|(_idx, val)| format!("{}:{}", val.0, val.1))
        .interleave(asks_list);

    let crc = Itertools::intersperse(crc_col, ":".to_string()).collect::<String>();
    let cs = crc32fast::hash(crc.as_bytes());
    let calc_cs = cs as i32;

    if calc_cs != checksum {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PriceQuantity;
    use rust_decimal_macros::dec;

    #[test]
    fn depth_validation() {
        // 1. Arrange
        let asks_raw = vec![
            PriceQuantity {
                price: dec!(8476.98),
                quantity: dec!(415),
            },
            PriceQuantity {
                price: dec!(8477),
                quantity: dec!(7),
            },
        ];
        let bids_raw = vec![
            PriceQuantity {
                price: dec!(8476.97),
                quantity: dec!(256),
            },
            PriceQuantity {
                price: dec!(8475.55),
                quantity: dec!(101),
            },
            PriceQuantity {
                price: dec!(8475.54),
                quantity: dec!(100),
            },
        ];

        // 2. Act
        let is_valid = is_checksum_valid(&asks_raw, &bids_raw, -1404728636);

        // 3. Assert
        assert!(is_valid);
    }
}
