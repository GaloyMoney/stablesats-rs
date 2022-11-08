#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod config;
mod convert;
pub mod error;
pub mod okex_shared;
pub mod order_book;
pub mod price_feed;

use futures::StreamExt;
use shared::{payload::*, pubsub::*};
use tokio::{join, time::Duration};

pub use config::*;
pub use error::*;
pub use okex_shared::*;
pub use order_book::*;
pub use price_feed::*;

pub async fn run(
    price_feed_config: PriceFeedConfig,
    pubsub_cfg: PubSubConfig,
) -> Result<(), PriceFeedError> {
    let publisher = Publisher::new(pubsub_cfg.clone()).await?;

    let ticks_publisher = publisher.clone();
    let pf_config = price_feed_config.clone();
    let mut stream = subscribe_btc_usd_swap_price_tick(pf_config).await?;

    let tick_task = tokio::spawn(async move {
        while let Some(tick) = stream.next().await {
            let _res = okex_price_tick_received(&ticks_publisher, tick).await;
        }
    });

    let order_book_task = tokio::spawn(async move {
        loop {
            let _res = order_book_subscription(publisher.clone(), &price_feed_config).await;
            tokio::time::sleep(Duration::from_secs(5_u64)).await;
        }
    });

    let _ = join!(tick_task, order_book_task);

    Ok(())
}

async fn order_book_subscription(
    publisher: Publisher,
    config: &PriceFeedConfig,
) -> Result<(), PriceFeedError> {
    let mut stream = subscribe_btc_usd_swap_order_book(config.clone()).await?;
    let full_load = stream.next().await.ok_or(PriceFeedError::InitialFullLoad)?;
    let order_book = CompleteOrderBook::try_from(OrderBookIncrement::try_from(full_load)?)?;
    let cache = OrderBookCache::new(order_book);

    let (send, recv) = tokio::sync::oneshot::channel();

    tokio::spawn(async move {
        while let Some(book) = stream.next().await {
            if let Err(e) = okex_order_book_received(&publisher, book, cache.clone()).await {
                let _ = send.send(e);
                break;
            }
        }
    });
    let _receiver = recv.await;
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
        if let Ok(complete_order_book) =
            OkexBtcUsdSwapOrderBookPayload::try_from(cache.latest().clone())
        {
            publisher.throttle_publish(complete_order_book).await?;
        }
    }

    Ok(())
}
