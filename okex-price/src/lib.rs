#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod convert;
mod price_feed;

use futures::StreamExt;

use shared::{payload::*, pubsub::*};

pub use price_feed::*;

pub async fn run(
    pubsub_cfg: PubSubConfig,
    price_feed_config: PriceFeedConfig,
) -> Result<(), PriceFeedError> {
    let publisher = Publisher::new(pubsub_cfg).await?;

    let mut stream = subscribe_btc_usd_swap(price_feed_config).await?;
    while let Some(tick) = stream.next().await {
        if let Ok(payload) = OkexBtcUsdSwapPricePayload::try_from(tick) {
            publisher.publish(payload).await?;
        }
    }

    Ok(())
}
