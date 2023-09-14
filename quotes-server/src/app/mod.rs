mod config;

use rust_decimal::Decimal;

use shared::{
    health::HealthCheckTrigger,
    payload::{PriceStreamPayload, BITFINEX_EXCHANGE_ID, OKEX_EXCHANGE_ID},
    pubsub::*,
};

use crate::{cache::*, currency_exchange::*, error::*, quote::*};
pub use config::*;

pub struct QuotesApp {}

impl QuotesApp {
    pub async fn run(
        health_check_trigger: HealthCheckTrigger,
        health_check_cfg: QuotesServerHealthCheckConfig,
        fee_calc_cfg: FeeCalculatorConfig,
        subscriber: memory::Subscriber<PriceStreamPayload>,
        price_cache_config: ExchangePriceCacheConfig,
        exchange_weights: ExchangeWeights,
    ) -> Result<Self, QuotesAppError> {
        Ok(Self {})
    }

    pub async fn quote_buy_cents_from_sats(
        &self,
        sats: Decimal,
        execute: bool,
    ) -> Result<Quote, QuotesAppError> {
        let quote = NewQuote::builder()
            .direction(Direction::BuyCents)
            .immediate_execution(execute)
            .build()
            .expect("Could not build quote");
        unimplemented!()
    }
}
