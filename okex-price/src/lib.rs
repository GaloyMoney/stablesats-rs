#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod config;
mod convert;
pub mod error;
pub mod okex_shared;
pub mod order_book;
pub mod price_tick;

use futures::{stream::SplitStream, SinkExt, Stream, StreamExt};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use shared::{payload::*, pubsub::*};

pub use config::*;
pub use error::*;
pub use okex_shared::*;
pub use order_book::*;
pub use price_tick::*;
use tokio::{
    net::TcpStream,
    sync::mpsc::{channel, Receiver},
};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

#[derive(Debug)]
pub enum Price {
    Tick(OkexPriceTick),
    Book(OrderBookIncrement),
}

#[derive(Debug, Serialize)]
struct SubscibeArgs {
    op: String,
    args: Vec<ChannelArgs>,
}

async fn subscribe_to_okex<T>(
    channels: Vec<ChannelArgs>,
    config: PriceFeedConfig,
) -> Result<Receiver<T>, PriceFeedError>
where
    T: DeserializeOwned + Clone + Send + 'static,
{
    let (ws_stream, _) = connect_async(config.url).await?;
    let (mut sender, mut receiver) = ws_stream.split();

    let subscribe_args = SubscibeArgs {
        op: "subscribe".to_string(),
        args: channels,
    };
    let subscribe_args = serde_json::to_string(&subscribe_args)?;

    let item = Message::Text(subscribe_args);
    sender.send(item).await?;

    let (tx1, rx1) = tokio::sync::mpsc::channel(100);

    let _jh = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            if let Ok(msg) = msg {
                if let Ok(msg_text) = msg.into_text() {
                    if let Ok(msg) = serde_json::from_str::<T>(&msg_text) {
                        let _send = tx1.send(msg).await;
                    }
                }
            }
        }
    });

    Ok(rx1)
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

    let mut stream = subscribe_btc_usd_swap_order_book(price_feed_config).await?;
    let full_load = stream.next().await.ok_or(PriceFeedError::InitialFullLoad)?;
    let full_incr = OrderBookIncrement::try_from(full_load)?;
    let cache = OrderBookCache::new(full_incr.try_into()?);

    handles.push(tokio::spawn(async move {
        while let Some(book) = stream.next().await {
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
