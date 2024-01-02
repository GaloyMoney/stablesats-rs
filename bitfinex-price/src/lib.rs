#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod config;
mod convert;
pub mod error;
pub mod price_feed;

use futures::StreamExt;
use shared::{payload::*, pubsub::*};
use tokio::join;

pub use config::*;
pub use price_feed::*;

pub async fn run(
    price_feed_config: PriceFeedConfig,
    price_stream_publisher: memory::Publisher<PriceStreamPayload>,
) -> Result<(), PriceFeedError> {
    let pf_config = price_feed_config.clone();
    let mut stream = subscribe_btc_usd_swap_price_tick(pf_config).await?;

    let tick_task = tokio::spawn(async move {
        while let Some(tick) = stream.next().await {
            let _res = bitfinex_price_tick_received(&price_stream_publisher, tick).await;
        }
    });
    let _ = join!(tick_task);

    Ok(())
}

async fn bitfinex_price_tick_received(
    publisher: &memory::Publisher<PriceStreamPayload>,
    tick: BitfinexPriceTick,
) -> Result<(), PriceFeedError> {
    if let Ok(payload) = PriceStreamPayload::try_from(tick) {
        publisher
            .throttle_publish("BITFINEX_PRICE_TICK", payload)
            .await?;
    }
    Ok(())
}
