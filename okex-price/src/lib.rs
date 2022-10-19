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
    let mut stream = subscribe_btc_usd_swap_order_book(price_feed_config).await?;

    let full_load = stream.next().await.ok_or(PriceFeedError::InitialFullLoad)?;

    let snapshot = CurrentSnapshot::new(full_load.try_into()?).await;
    while let Some(payload) = stream.next().await {
        let incr: CurrentSnapshotInner = payload.try_into()?;

        snapshot.merge(incr).await;
        let _ = publisher.throttle_publish::<OkexBtcUsdSwapOrderBookPayload>(
            snapshot.latest_snapshot().await.into(),
        );
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
