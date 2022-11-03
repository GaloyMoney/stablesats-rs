use super::price_feed::error::KolliderPriceFeedError;
use super::price_feed::KolliderPriceTicker;

use shared::{
    payload::{
        ExchangeIdRaw, InstrumentIdRaw, KolliderBtcUsdSwapPricePayload, PriceMessagePayload,
        PriceRatioRaw, KOLLIDER_EXCHANGE_ID,
    },
    time::TimeStamp,
};

impl TryFrom<KolliderPriceTicker> for KolliderBtcUsdSwapPricePayload {
    type Error = KolliderPriceFeedError;
    fn try_from(value: KolliderPriceTicker) -> Result<Self, Self::Error> {
        Ok(KolliderBtcUsdSwapPricePayload(PriceMessagePayload {
            exchange: ExchangeIdRaw::from(KOLLIDER_EXCHANGE_ID),
            instrument_id: InstrumentIdRaw::from(value.symbol),
            timestamp: TimeStamp::now(),
            ask_price: PriceRatioRaw::from_one_btc_in_usd_price(value.best_ask),
            bid_price: PriceRatioRaw::from_one_btc_in_usd_price(value.best_bid),
        }))
    }
}
