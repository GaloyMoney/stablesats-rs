use std::io::Error;

use futures::StreamExt;
pub use price_feed::*;
use shared::{
    payload::KolliderBtcUsdSwapPricePayload,
    pubsub::{PubSubConfig, Publisher},
};
mod price_feed;

mod convert;

pub async fn run(pubsub_cfg: PubSubConfig) -> Result<(), Error> {
    let publisher = Publisher::new(pubsub_cfg).await.unwrap(); //FIXME

    let mut stream = subscribe_price_feed().await.unwrap(); // FIXME
    while let Some(tick) = stream.next().await {
        //let _ = okex_price_tick_received(&publisher, tick).await;

        println!("publish payload {:?}", tick);
        if let Ok(payload) = KolliderBtcUsdSwapPricePayload::try_from(tick) {
            publisher.throttle_publish(payload).await.unwrap();
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::price_feed::subscribe_price_feed;
    use futures::StreamExt;

    #[tokio::test]
    async fn test_get_price() {
        let mut stream = subscribe_price_feed().await.unwrap();
        while let Some(tick) = stream.next().await {
            println!("result: {:?}", tick);
        }
    }

    #[tokio::test]
    async fn test_it() {
        println!("hhmm");
    }
}
