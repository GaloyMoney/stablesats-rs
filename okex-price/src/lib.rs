#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod convert;
pub mod error;
pub mod okex_shared;
pub mod order_book;
pub mod price_feed;

use futures::StreamExt;
use shared::{payload::*, pubsub::*};
use tokio::{join, time::timeout};

pub use okex_shared::*;
pub use order_book::*;
pub use price_feed::*;

pub async fn run(
    price_stream_publisher: memory::Publisher<PriceStreamPayload>,
    unhealthy_msg_interval: std::time::Duration,
) -> Result<(), PriceFeedError> {
    let _ = tokio::spawn(async move {
        loop {
            let tick_publisher = price_stream_publisher.clone();
            if let Ok(mut stream) = subscribe_btc_usd_swap_price_tick().await {
                let tick_task = tokio::spawn(async move {
                    while let Some(tick) = stream.next().await {
                        let _res = okex_price_tick_received(&tick_publisher, tick).await;
                    }
                });
                let order_book_publisher = price_stream_publisher.clone();
                let order_book_task = tokio::spawn(async move {
                    loop {
                        let _res = order_book_subscription(
                            order_book_publisher.clone(),
                            unhealthy_msg_interval,
                        )
                        .await;
                        tokio::time::sleep(std::time::Duration::from_secs(5_u64)).await;
                    }
                });
                let _ = join!(tick_task, order_book_task);
            }
        }
    })
    .await;

    Ok(())
}

async fn order_book_subscription(
    publisher: memory::Publisher<PriceStreamPayload>,
    unhealthy_msg_interval: std::time::Duration,
) -> Result<(), PriceFeedError> {
    let mut stream = subscribe_btc_usd_swap_order_book().await?;
    let full_load = stream.next().await.ok_or(PriceFeedError::InitialFullLoad)?;
    let order_book = CompleteOrderBook::try_from(OrderBookIncrement::try_from(full_load)?)?;
    let mut cache = OrderBookCache::new(order_book);

    let (send, recv) = tokio::sync::oneshot::channel();

    tokio::spawn(async move {
        loop {
            match timeout(unhealthy_msg_interval, stream.next()).await {
                Ok(Some(book)) => {
                    if let Err(e) = okex_order_book_received(&publisher, book, &mut cache).await {
                        let _ = send.send(e);
                        break;
                    }
                }
                Ok(None) => {
                    let _ = send.send(PriceFeedError::StreamEnded);
                    break;
                }
                Err(_) => {
                    let _ = send.send(PriceFeedError::StreamStalled);
                    break;
                }
            }
        }
    });
    let _receiver = recv.await;
    Ok(())
}

async fn okex_price_tick_received(
    publisher: &memory::Publisher<PriceStreamPayload>,
    tick: OkexPriceTick,
) -> Result<(), PriceFeedError> {
    if let Ok(payload) = PriceStreamPayload::try_from(tick) {
        publisher
            .throttle_publish("OKEX_PRICE_TICK", payload)
            .await?;
    }
    Ok(())
}

async fn okex_order_book_received(
    publisher: &memory::Publisher<PriceStreamPayload>,
    book: OkexOrderBook,
    cache: &mut OrderBookCache,
) -> Result<(), PriceFeedError> {
    if let Ok(increment) = OrderBookIncrement::try_from(book) {
        cache.update_order_book(increment)?;
        if let Ok(complete_order_book) = OrderBookPayload::try_from(cache.latest().clone()) {
            publisher
                .throttle_publish(
                    "OKEX_ORDER_BOOK",
                    PriceStreamPayload::OkexBtcUsdSwapOrderBookPayload(complete_order_book),
                )
                .await?;
        }
    }

    Ok(())
}
