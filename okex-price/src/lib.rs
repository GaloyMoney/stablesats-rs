#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod config;
mod convert;
pub mod error;
pub mod okex_shared;
pub mod order_book;
pub mod price_tick;

use futures::StreamExt;
use shared::{payload::*, pubsub::*};

pub use config::*;
pub use error::*;
pub use okex_shared::*;
pub use order_book::*;
pub use price_tick::*;
use tokio::sync::mpsc::channel;

pub enum Price {
    Tick(OkexPriceTick),
    Book(OkexOrderBook),
}

pub async fn run(
    price_feed_config: PriceFeedConfig,
    pubsub_cfg: PubSubConfig,
) -> Result<(), PriceFeedError> {
    let publisher = Publisher::new(pubsub_cfg.clone()).await?;
    let (tx, mut rx) = channel(100);

    let tx1 = tx.clone();
    let mut tick_stream = subscribe_btc_usd_swap_price_tick(price_feed_config.clone()).await?;
    let _ticks = tokio::spawn(async move {
        while let Some(tick) = tick_stream.next().await {
            let _ = tx1.send(Price::Tick(tick)).await;
        }
    });

    let mut book_stream = subscribe_btc_usd_swap_order_book(price_feed_config.clone()).await?;
    let initial_full_load = book_stream
        .next()
        .await
        .ok_or(PriceFeedError::InitialFullLoad)?;
    let snapshot = CurrentSnapshot::new(initial_full_load.try_into()?).await;
    let _books = tokio::spawn(async move {
        while let Some(book) = book_stream.next().await {
            let _ = tx.send(Price::Book(book)).await;
        }
    });

    let _publishing = tokio::spawn(async move {
        while let Some(price) = rx.recv().await {
            match price {
                Price::Tick(tick) => {
                    let _ = okex_price_tick_received(&publisher, tick).await;
                }
                Price::Book(book) => {
                    let _ = okex_order_book_received(&publisher, book, &snapshot).await;
                }
            }
        }
    });

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
    snapshot: &CurrentSnapshot,
) -> Result<(), PriceFeedError> {
    if let Ok(payload) = CurrentSnapshotInner::try_from(book) {
        snapshot.merge(payload).await;
        let _ = publisher.throttle_publish::<OkexBtcUsdSwapOrderBookPayload>(
            snapshot.latest_snapshot().await.into(),
        );
    }

    Ok(())
}
