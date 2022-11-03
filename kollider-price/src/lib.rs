use futures::StreamExt;
mod price_feed;
use price_feed::config::KolliderPriceFeedConfig;
pub use price_feed::*;
use shared::{
    payload::KolliderBtcUsdSwapPricePayload,
    pubsub::{PubSubConfig, Publisher},
};

pub use price_feed::error::KolliderPriceFeedError;

mod convert;

pub async fn run(
    price_feed_config: KolliderPriceFeedConfig,
    pubsub_cfg: PubSubConfig,
) -> Result<(), KolliderPriceFeedError> {
    let publisher = Publisher::new(pubsub_cfg).await?;

    let mut stream = subscribe_price_feed(price_feed_config).await?;
    while let Some(tick) = stream.next().await {
        println!("publish payload {:?}", tick);
        if let Ok(payload) = KolliderBtcUsdSwapPricePayload::try_from(tick) {
            publisher.throttle_publish(payload).await?;
        }
    }

    Ok(())
}
