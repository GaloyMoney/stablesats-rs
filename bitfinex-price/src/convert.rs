use crate::config::*;
use shared::{payload::*, time::*};

use super::price_feed::{BitfinexPriceTick, PriceFeedError};

impl TryFrom<BitfinexPriceTick> for PriceStreamPayload {
    type Error = PriceFeedError;

    fn try_from(BitfinexPriceTick { tick, .. }: BitfinexPriceTick) -> Result<Self, Self::Error> {
        Ok(PriceStreamPayload::BitfinexBtcUsdSwapPricePayload(
            PriceMessagePayload {
                exchange: ExchangeIdRaw::from(BITFINEX_EXCHANGE_ID),
                instrument_id: InstrumentIdRaw::from(BTC_USD_SWAP),
                timestamp: TimeStamp::now(),
                ask_price: PriceRatioRaw::from_one_btc_in_usd_price(tick.ask),
                bid_price: PriceRatioRaw::from_one_btc_in_usd_price(tick.bid),
            },
        ))
    }
}
