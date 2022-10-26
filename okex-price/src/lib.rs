#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod config;
mod convert;
pub mod error;
pub mod okex_shared;
pub mod order_book;
pub mod price_tick;

use std::pin::Pin;

use futures::{SinkExt, Stream, StreamExt};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use shared::{payload::*, pubsub::*};

pub use config::*;
pub use error::*;
pub use okex_shared::*;
pub use order_book::*;
pub use price_tick::*;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[derive(Deserialize, Clone)]
// #[serde(untagged)]
pub enum OkexPriceData {
    Tick(OkexPriceTick),
    Book(OkexOrderBook),
}

#[derive(Debug, Serialize)]
struct SubscibeArgs {
    op: String,
    args: Vec<ChannelArgs>,
}

pub async fn subscribe_to_okex<T>(
    channels: Vec<ChannelArgs>,
    config: PriceFeedConfig,
) -> Result<Pin<Box<dyn Stream<Item = T> + Send>>, PriceFeedError>
where
    T: DeserializeOwned + Sized + Clone + Send + 'static,
{
    let (ws_stream, _) = connect_async(config.url).await?;
    let (mut sender, receiver) = ws_stream.split();

    let subscribe_args = SubscibeArgs {
        op: "subscribe".to_string(),
        args: channels,
    };
    let subscribe_args = serde_json::to_string(&subscribe_args)?;

    let item = Message::Text(subscribe_args);
    sender.send(item).await?;

    Ok(Box::pin(receiver.filter_map(|message| async {
        if let Ok(msg) = message {
            if let Ok(msg_str) = msg.into_text() {
                if let Ok(book) = serde_json::from_str::<T>(&msg_str) {
                    return Some(book);
                }
            }
        }
        None
    })))
}

pub async fn run(
    price_feed_config: PriceFeedConfig,
    pubsub_cfg: PubSubConfig,
) -> Result<(), PriceFeedError> {
    let publisher = Publisher::new(pubsub_cfg.clone()).await?;

    let ticks_publisher = publisher.clone();
    let pf_config = price_feed_config.clone();
    let mut stream = subscribe_btc_usd_swap_price_tick(pf_config).await?;

    let mut handles = Vec::new();
    handles.push(tokio::spawn(async move {
        while let Some(tick) = stream.next().await {
            let _res = okex_price_tick_received(&ticks_publisher, tick).await;
        }
    }));

    let mut stream = subscribe_btc_usd_swap_order_book(price_feed_config.clone()).await?;
    let full_load = stream.next().await.ok_or(PriceFeedError::InitialFullLoad)?;
    let full_incr = OrderBookIncrement::try_from(full_load)?;
    let cache = OrderBookCache::new(full_incr.try_into()?);

    handles.push(tokio::spawn(async move {
        while let Some(book) = stream.next().await {
            // std::fs::write("incr.json", serde_json::to_string_pretty(&book).unwrap()).unwrap();

            let _res = okex_order_book_received(&publisher, book, cache.clone()).await;
        }
    }));

    for handle in handles {
        let _res = handle.await;
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

async fn okex_order_book_received(
    publisher: &Publisher,
    book: OkexOrderBook,
    mut cache: OrderBookCache,
) -> Result<(), PriceFeedError> {
    if let Ok(increment) = OrderBookIncrement::try_from(book) {
        cache.update_order_book(increment)?;
        if let Ok(complete_order_book) = OkexBtcUsdSwapOrderBookPayload::try_from(cache.latest()) {
            publisher
                .throttle_publish::<OkexBtcUsdSwapOrderBookPayload>(complete_order_book)
                .await?;
        }
    }

    Ok(())
}
