use crate::config::*;
use shared::{payload::*, time::*};

use super::price_feed::{DeribitPriceTick, PriceFeedError};

impl TryFrom<DeribitPriceTick> for PriceStreamPayload {
    type Error = PriceFeedError;

    fn try_from(DeribitPriceTick { params, .. }: DeribitPriceTick) -> Result<Self, Self::Error> {
        Ok(PriceStreamPayload::DeribitBtcUsdSwapPricePayload(
            PriceMessagePayload {
                exchange: ExchangeIdRaw::from(DERIBIT_EXCHANGE_ID),
                instrument_id: InstrumentIdRaw::from(BTC_USD_SWAP),
                timestamp: TimeStamp::now(),
                ask_price: PriceRatioRaw::from_one_btc_in_usd_price(params.data.best_ask_price),
                bid_price: PriceRatioRaw::from_one_btc_in_usd_price(params.data.best_bid_price),
            },
        ))
    }
}
