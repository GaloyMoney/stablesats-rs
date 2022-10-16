#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod config;
mod convert;
pub mod error;
pub mod okex_shared;
pub mod order_book;
pub mod price_feed;

use std::pin::Pin;

use futures::{Stream, StreamExt};
use shared::{payload::*, pubsub::*};

pub use config::*;
pub use error::*;
pub use okex_shared::*;
pub use order_book::*;
pub use price_feed::*;
use tokio::sync::mpsc::Sender;

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

pub async fn run_book(
    price_feed_config: PriceFeedConfig,
    pubsub_cfg: PubSubConfig,
) -> Result<(), PriceFeedError> {
    let publisher = Publisher::new(pubsub_cfg).await?;
    let stream = OrderBook::subscribe(price_feed_config).await?;
    let (sender, mut receiver) = tokio::sync::mpsc::channel(1);

    tokio::spawn(convert_to_payload(sender.clone(), stream));

    while let Some(payload) = receiver.recv().await {
        let update_payload = payload;

        let snapshot = OkexBtcUsdSwapOrderBookPayload::merge(update_payload).await?;
        let _ = publisher.throttle_publish(snapshot);
    }

    Ok(())
}

async fn okex_price_tick_received(
    publisher: &Publisher,
    tick: OkexPriceTick,
) -> Result<(), PriceFeedError> {
    if let Ok(payload) = OkexBtcUsdSwapPricePayload::try_from(tick) {
        publisher.throttle_publish(payload).await?;
    }
    Ok(())
}

/// Convert OkexOrderBook to OkexBtcUsdSwapOrderBookPayload and send to channel
async fn convert_to_payload(
    sender: Sender<OkexBtcUsdSwapOrderBookPayload>,
    mut stream: Pin<Box<dyn Stream<Item = OkexOrderBook> + Send>>,
) -> Result<(), PriceFeedError> {
    while let Some(book) = stream.next().await {
        let payload = OkexBtcUsdSwapOrderBookPayload::try_from(book)?;
        let _ = sender.try_send(payload);
    }

    Ok(())
}
