#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod convert;
mod price_feed;

use futures::StreamExt;
use tracing::instrument;

use shared::{payload::*, pubsub::*};

pub use price_feed::*;

pub async fn run(
    price_feed_config: PriceFeedConfig,
    pubsub_cfg: PubSubConfig,
) -> Result<(), PriceFeedError> {
    let publisher = Publisher::new(pubsub_cfg).await?;

    let mut stream = subscribe_btc_usd_swap(price_feed_config).await?;
    while let Some(tick) = stream.next().await {
        let _ = okex_price_tick_received(&publisher, tick).await;
    }

    Ok(())
}

#[instrument(skip(publisher), err)]
async fn okex_price_tick_received(
    publisher: &Publisher,
    tick: OkexPriceTick,
) -> Result<(), PriceFeedError> {
    if let Ok(payload) = OkexBtcUsdSwapPricePayload::try_from(tick) {
        publisher.throttle_publish(payload).await?
    }
    Ok(())
}
