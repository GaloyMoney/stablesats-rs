mod convert;
mod error;
mod price_feed;

use futures::StreamExt;
use price_feed::config::KolliderPriceFeedConfig;
pub use price_feed::*;
use shared::{payload::PriceStreamPayload, pubsub::*};

pub use error::PriceFeedError;

pub async fn run(
    price_feed_config: KolliderPriceFeedConfig,
    price_stream_publisher: memory::Publisher<PriceStreamPayload>,
) -> Result<(), PriceFeedError> {
    let mut stream = subscribe_price_feed(price_feed_config).await?;
    while let Some(tick) = stream.next().await {
        if let Ok(payload) = PriceStreamPayload::try_from(tick) {
            price_stream_publisher
                .throttle_publish("KOLLIDER_PRICE", payload)
                .await?;
        }
    }

    Ok(())
}
