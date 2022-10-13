#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod config;
mod convert;
pub mod okex_shared;
pub mod order_book;
pub mod price_feed;

use std::pin::Pin;

use anyhow::Context;
use futures::{channel::mpsc::unbounded, Stream, StreamExt};
use shared::{payload::*, pubsub::*};

pub use config::*;
pub use okex_shared::*;
pub use order_book::*;
pub use price_feed::*;

pub type OkexSnapshotStream = Pin<Box<dyn Stream<Item = OkexBtcUsdSwapOrderBookPayload> + Send>>;
pub type OkexPriceTickStream = Pin<Box<dyn Stream<Item = OkexBtcUsdSwapPricePayload> + Send>>;

struct PriceMatch {
    incr_data: (usize, PriceQuantity),
    snap_data: (usize, PriceQuantity),
}

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
) -> Result<(), OrderBookError> {
    let publisher = Publisher::new(pubsub_cfg).await?;
    let mut stream = OrderBook::subscribe(price_feed_config).await?;
    let (sender, mut receiver) = tokio::sync::mpsc::channel(1);
    // 1. Task to merge incremental updates in local full load snapshot of the order book
    let merge_joinhandle = tokio::spawn(async move {
        while let Some(book) = stream.next().await {
            let merge_res = merge_update(book)
                .await
                .context("Error merging incremental order book updates");
        }
    });

    // 2. Publish snapshot to redis
    while let Some(snapshot) = receiver.recv().await {
        let _ = okex_snapshot_received(&publisher, snapshot);
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

async fn merge_update(
    book: OkexOrderBook,
) -> Result<OkexBtcUsdSwapOrderBookPayload, PriceFeedError> {
    let mut local_full_load;
    if book.action == OrderBookAction::Snapshot {
        local_full_load = book;
    }

    match book.action {
        OrderBookAction::Snapshot => OkexBtcUsdSwapOrderBookPayload::try_from(book),
        OrderBookAction::Update => {
            let price_matches = same_price(book, &local_full_load)?;
            if price_matches.len() > 0 {
                if size_is_zero() {
                    // delete data
                    todo!()
                } else {
                    // replace original depth data
                    todo!()
                }
            } else {
                sort_insert()
            }
        }
    }

    unimplemented!()
}

async fn okex_snapshot_received(
    publisher: &Publisher,
    snapshot: OkexBtcUsdSwapOrderBookPayload,
) -> Result<(), OrderBookError> {
    unimplemented!()
}

/// Checks if the snapshot shot and update have the same price
fn same_price(
    update: OkexOrderBook,
    snapshot: &OkexOrderBook,
) -> Result<Vec<PriceMatch>, OrderBookError> {
    let price_matches = Vec::new();

    let (update_data, snapshot_data) = (
        update.data.first().ok_or(OrderBookError::EmptyOrderBook)?,
        snapshot
            .data
            .first()
            .ok_or(OrderBookError::EmptyOrderBook)?,
    );

    for (incr_idx, incr_price_quantity) in update_data.asks.iter().enumerate() {
        for (snap_idx, snap_price_quantity) in snapshot_data.asks.iter().enumerate() {
            if *incr_price_quantity == *snap_price_quantity {
                price_matches.push(PriceMatch {
                    incr_data: (incr_idx, *incr_price_quantity),
                    snap_data: (snap_idx, *snap_price_quantity),
                })
            }
        }
    }

    Ok(price_matches)
}

/// Checks if the size(i.e. quantity at a given depth) in the update is the same as in the snapshot
/// for data points with the same price
fn size_is_zero() -> bool {
    unimplemented!()
}

/// Sort the bids (descending) and asks (ascending), and then insert update
fn sort_insert() {
    unimplemented!()
}
