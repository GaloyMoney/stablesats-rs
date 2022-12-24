#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod config;
mod convert;
pub mod error;
pub mod okex_shared;
pub mod order_book;
pub mod price_feed;

use futures::StreamExt;
use shared::{payload::*, pubsub::*, time::TimeStamp};
use tokio::join;

pub use config::*;
pub use error::*;
pub use okex_shared::*;
pub use order_book::*;
pub use price_feed::*;

pub async fn run(
    price_feed_config: PriceFeedConfig,
    ticks_publisher: memory::Publisher<OkexBtcUsdSwapPricePayload>,
) -> Result<(), PriceFeedError> {
    let pf_config = price_feed_config.clone();
    let mut stream = subscribe_btc_usd_swap_price_tick(pf_config).await?;

    let tick_task = if let Some(price) = price_feed_config.dev_mock_price_btc_in_usd {
        tokio::spawn(async move {
            loop {
                ticks_publisher
                    .publish(OkexBtcUsdSwapPricePayload(PriceMessagePayload {
                        timestamp: TimeStamp::now(),
                        exchange: ExchangeIdRaw::from(OKEX_EXCHANGE_ID),
                        instrument_id: InstrumentIdRaw::from("BTC-USD-SWAP".to_string()),
                        ask_price: PriceRatioRaw::from_one_btc_in_usd_price(price),
                        bid_price: PriceRatioRaw::from_one_btc_in_usd_price(price),
                    }))
                    .await
                    .expect("failed to publish tick");
                tokio::time::sleep(
                    price_feed_config
                        .rate_limit_interval
                        .to_std()
                        .expect("rate limit is not positive"),
                )
                .await;
            }
        })
    } else {
        tokio::spawn(async move {
            while let Some(tick) = stream.next().await {
                let _res = okex_price_tick_received(&ticks_publisher, tick).await;
            }
        })
    };

    // let order_book_task = tokio::spawn(async move {
    //     loop {
    //         let _res = order_book_subscription(books_publisher.clone(), &price_feed_config).await;
    //         tokio::time::sleep(Duration::from_secs(5_u64)).await;
    //     }
    // });

    // let _ = join!(tick_task, order_book_task);
    let _ = join!(tick_task);

    Ok(())
}

#[allow(dead_code)]
async fn order_book_subscription(
    publisher: memory::Publisher<OkexBtcUsdSwapOrderBookPayload>,
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
    publisher: &memory::Publisher<OkexBtcUsdSwapPricePayload>,
    tick: OkexPriceTick,
) -> Result<(), PriceFeedError> {
    if let Ok(payload) = OkexBtcUsdSwapPricePayload::try_from(tick) {
        publisher.throttle_publish(payload).await?;
    }
    Ok(())
}

#[allow(dead_code)]
async fn okex_order_book_received(
    publisher: &memory::Publisher<OkexBtcUsdSwapOrderBookPayload>,
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
