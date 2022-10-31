use std::io::Error;

use futures::StreamExt;
mod price_feed;
use price_feed::config::KolliderPriceFeedConfig;
pub use price_feed::*;
use shared::{
    payload::KolliderBtcUsdSwapPricePayload,
    pubsub::{PubSubConfig, Publisher},
};

mod convert;

pub async fn run(
    price_feed_config: KolliderPriceFeedConfig,
    pubsub_cfg: PubSubConfig,
) -> Result<(), Error> {
    let publisher = Publisher::new(pubsub_cfg).await.unwrap(); //FIXME

    let mut stream = subscribe_price_feed(price_feed_config).await.unwrap(); // FIXME
    while let Some(tick) = stream.next().await {
        println!("publish payload {:?}", tick);
        if let Ok(payload) = KolliderBtcUsdSwapPricePayload::try_from(tick) {
            publisher.throttle_publish(payload).await.unwrap();
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{config::KolliderPriceFeedConfig, price_feed::subscribe_price_feed};
    use futures::StreamExt;

    #[tokio::test]
    async fn test_get_price() {
        let config = KolliderPriceFeedConfig::default();
        let mut stream = subscribe_price_feed(config).await.unwrap();
        while let Some(tick) = stream.next().await {
            println!("result: {:?}", tick);
        }
    }

    #[tokio::test]
    async fn test_it() {
        println!("hhmm");
    }
}
