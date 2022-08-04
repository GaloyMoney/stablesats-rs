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
