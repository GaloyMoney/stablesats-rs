use super::error::PriceFeedError;
use super::price_feed::KolliderPriceTicker;

use shared::{
    payload::{
        ExchangeIdRaw, InstrumentIdRaw, PriceMessagePayload, PriceRatioRaw, PriceStreamPayload,
        KOLLIDER_EXCHANGE_ID,
    },
    time::TimeStamp,
};

impl TryFrom<KolliderPriceTicker> for PriceStreamPayload {
    type Error = PriceFeedError;
    fn try_from(value: KolliderPriceTicker) -> Result<Self, Self::Error> {
        Ok(PriceStreamPayload::KolliderBtcUsdSwapPricePayload(
            PriceMessagePayload {
                exchange: ExchangeIdRaw::from(KOLLIDER_EXCHANGE_ID),
                instrument_id: InstrumentIdRaw::from(value.symbol),
                timestamp: TimeStamp::now(),
                ask_price: PriceRatioRaw::from_one_btc_in_usd_price(value.best_ask),
                bid_price: PriceRatioRaw::from_one_btc_in_usd_price(value.best_bid),
            },
        ))
    }
}
