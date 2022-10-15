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

impl From<OrderBookAction> for OrderBookActionRaw {
    fn from(action: OrderBookAction) -> Self {
        match action {
            OrderBookAction::Snapshot => Self::Snapshot,
            OrderBookAction::Update => Self::Update,
        }
    }
}

impl TryFrom<OkexOrderBook> for OkexBtcUsdSwapOrderBookPayload {
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

        let asks_tups = asks
            .iter()
            .map(|price_quantity| (price_quantity.price, price_quantity.quantity))
            .collect::<Vec<(_, _)>>();
        let bids_tups = bids
            .iter()
            .map(|price_quantity| (price_quantity.price, price_quantity.quantity))
            .collect::<Vec<(_, _)>>();

        Ok(OkexBtcUsdSwapOrderBookPayload(OrderBookPayload {
            asks: OrderBookRaw::from_okex_order_book(asks_tups),
            bids: OrderBookRaw::from_okex_order_book(bids_tups),
            timestamp: TimeStamp::try_from(ts)?,
            checksum: CheckSumRaw::from(*checksum),
            action: OrderBookActionRaw::from(action),
        }))
    }
}
