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

        // TODO: Checksum validation
        // if !is_checksum_valid(asks, bids, *checksum) {
        //     return Err(PriceFeedError::CheckSumValidation);
        // }

        let inner = CurrentSnapshotInner {
            asks: PriceQtyMap::from(asks),
            bids: PriceQtyMap::from(bids),
            timestamp: TimeStamp::try_from(ts)?,
            checksum: CheckSum::from(*checksum),
            action,
        };
        Ok(inner)
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
fn _is_checksum_valid(asks: &[PriceQuantity], bids: &[PriceQuantity], checksum: i32) -> bool {
    // 1. [x] Get the first 25 elements (if asks.len() > 25 and bids.len() > 25) or all elements
    // 2. [x] Create a vector of strings with price:quantity pair for both asks and bids
    // 2. [x] Use itertools to interleave the vectors into a new collection (string)
    // 3. [x] CRC32 the resulting string

    let asks_25_or_less = (*asks)
        // .clone()
        .iter()
        .enumerate()
        .filter(|(idx, _)| idx < &OKEX_CHECKSUM_LIMIT)
        .map(|(_, val)| format!("{}:{}", val.price, val.quantity))
        .collect::<Vec<_>>();
    let crc_collection = (*bids)
        // .clone()
        .iter()
        .enumerate()
        .filter(|(idx, _)| idx < &OKEX_CHECKSUM_LIMIT)
        .map(|(_, val)| format!("{}:{}", val.price, val.quantity))
        .interleave(asks_25_or_less);

    // let crc_collection = bids_25_or_less.into_iter().interleave(asks_25_or_less);
    let crc = Itertools::intersperse(crc_collection, ":".to_string()).collect::<String>();
    let checksum_val = crc32fast::hash(crc.as_bytes());
    let calc_cs = checksum_val as i32;

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
                price: dec!(19162.8),
                quantity: dec!(113),
            },
            PriceQuantity {
                price: dec!(19163.3),
                quantity: dec!(0),
            },
            PriceQuantity {
                price: dec!(19164.9),
                quantity: dec!(30),
            },
            PriceQuantity {
                price: dec!(19165.7),
                quantity: dec!(220),
            },
            PriceQuantity {
                price: dec!(19166.2),
                quantity: dec!(0),
            },
            PriceQuantity {
                price: dec!(19168.2),
                quantity: dec!(120),
            },
            PriceQuantity {
                price: dec!(19168.8),
                quantity: dec!(0),
            },
            PriceQuantity {
                price: dec!(19172),
                quantity: dec!(169),
            },
            PriceQuantity {
                price: dec!(19172.4),
                quantity: dec!(0),
            },
            PriceQuantity {
                price: dec!(19833),
                quantity: dec!(20),
            },
            PriceQuantity {
                price: dec!(19848),
                quantity: dec!(59),
            },
            PriceQuantity {
                price: dec!(19850),
                quantity: dec!(422),
            },
        ];
        let bids_raw = vec![
            PriceQuantity {
                price: dec!(19151.8),
                quantity: dec!(12),
            },
            PriceQuantity {
                price: dec!(19152.4),
                quantity: dec!(321),
            },
        ];

        // 2. Act
        let is_valid = _is_checksum_valid(&asks_raw, &bids_raw, 1009924713);

        // 3. Assert
        assert!(is_valid);
    }
}
