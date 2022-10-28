use super::price_feed::error::KolliderPriceFeedError;
use super::price_feed::KolliderPriceTicker;

use shared::{
    payload::{
        ExchangeIdRaw, InstrumentIdRaw, KolliderBtcUsdSwapPricePayload, PriceMessagePayload,
        PriceRatioRaw,
    },
    time::TimeStamp,
};

impl TryFrom<KolliderPriceTicker> for KolliderBtcUsdSwapPricePayload {
    type Error = KolliderPriceFeedError;
    fn try_from(value: KolliderPriceTicker) -> Result<Self, Self::Error> {
        Ok(KolliderBtcUsdSwapPricePayload(PriceMessagePayload {
            exchange: ExchangeIdRaw::from("Kollider"), // FIXME
            instrument_id: InstrumentIdRaw::from("BTC-USD-SWAP"), // FIXME "BTC-USD-SWAP"
            timestamp: TimeStamp::now(),               //FIXME
            ask_price: PriceRatioRaw::from_one_btc_in_usd_price(value.best_ask),
            bid_price: PriceRatioRaw::from_one_btc_in_usd_price(value.best_bid),
        }))
    }
}
