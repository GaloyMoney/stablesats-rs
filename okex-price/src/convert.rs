use std::collections::BTreeMap;

use crate::*;
use shared::{payload::*, time::*};

use super::price_feed::{OkexPriceTick, PriceFeedError};

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

impl TryFrom<OkexOrderBook> for OrderBookIncrement {
    type Error = PriceFeedError;

    fn try_from(value: OkexOrderBook) -> Result<Self, Self::Error> {
        let okex_order_book_data = value
            .data
            .first()
            .ok_or(PriceFeedError::EmptyOrderBookData)?;
        let action = value.action;

        let (asks, bids, ts, checksum) = (
            &okex_order_book_data.asks,
            &okex_order_book_data.bids,
            &okex_order_book_data.ts,
            &okex_order_book_data.checksum,
        );

        let mut asks_map = BTreeMap::new();
        for ask in asks {
            let _ = asks_map.insert(OrderPrice::from(ask.price), ask.quantity);
        }
        let mut bids_map = BTreeMap::new();
        for bid in bids {
            let _ = bids_map.insert(OrderPrice::from(bid.price), bid.quantity);
        }

        let inner = OrderBookIncrement {
            asks: asks_map,
            bids: bids_map,
            timestamp: TimeStamp::try_from(ts)?,
            new_checksum: *checksum,
            action,
        };
        Ok(inner)
    }
}

impl From<CompleteOrderBook> for OkexBtcUsdSwapOrderBookPayload {
    fn from(book: CompleteOrderBook) -> Self {
        Self(OrderBookPayload::from(book))
    }
}
