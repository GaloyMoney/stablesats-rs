mod error;

use futures::stream::StreamExt;

use super::exchange_price_cache::ExchangePriceCache;
pub use error::*;
use shared::{currency::*, payload::OkexBtcUsdSwapPricePayload, pubsub::*};

pub struct PriceApp {
    price_cache: ExchangePriceCache,
}

impl PriceApp {
    pub async fn run(pubsub_cfg: PubSubConfig) -> Result<Self, PriceAppError> {
        let subscriber = Subscriber::new(pubsub_cfg).await?;
        let mut stream = subscriber.subscribe::<OkexBtcUsdSwapPricePayload>().await?;

        let price_cache = ExchangePriceCache::new();
        let app = Self {
            price_cache: price_cache.clone(),
        };
        let _ = tokio::spawn(async move {
            while let Some(msg) = stream.next().await {
                let payload = msg.payload;
                let _ = price_cache.apply_update(payload).await;
            }
        });
        Ok(app)
    }

    pub async fn get_cents_from_sats_for_immediate_buy(
        &self,
        sats: impl Into<Sats>,
    ) -> Result<u64, PriceAppError> {
        unimplemented!()
    }
}
