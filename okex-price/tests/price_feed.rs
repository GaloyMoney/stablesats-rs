use chrono::Duration;
use futures::StreamExt;
use okex_price::*;
use std::fs;

use shared::{payload::*, pubsub::*, time::*};

#[derive(serde::Deserialize)]
struct Fixture {
    payloads: Vec<PriceMessagePayload>,
}

fn load_fixture() -> anyhow::Result<Fixture> {
    let contents =
        fs::read_to_string("./tests/fixtures/price_feed.json").expect("Couldn't load fixtures");
    Ok(serde_json::from_str(&contents)?)
}

#[tokio::test]
async fn subscribes_to_tickers_channel() -> anyhow::Result<()> {
    let config = PriceFeedConfig::default();
    let mut received = subscribe_btc_usd_swap_price_tick(config)
        .await
        .expect("subscribe_btc_usd_swap");
    let price_tick = received.next().await.expect("expected price tick");

    assert_eq!(
        price_tick.arg,
        ChannelArgs {
            channel: "tickers".to_string(),
            inst_id: "BTC-USD-SWAP".to_string(),
        }
    );
    assert_eq!(price_tick.data.len(), 1);
    let duration_since = TimeStamp::try_from(&price_tick.data[0].ts)
        .expect("couldn't convert timestamp")
        .duration_since();
    assert!(duration_since < Duration::seconds(30));

    let data = &price_tick.data[0];
    assert!(data.ask_px >= data.bid_px);
    Ok(())
}

#[tokio::test]
async fn subscribe_to_order_book_channel() -> anyhow::Result<()> {
    let config = PriceFeedConfig::default();
    let mut order_book_stream = subscribe_btc_usd_swap_order_book(config)
        .await
        .expect("subscribe to order book channel");
    let order_book = order_book_stream.next().await.expect("order book");

    assert_eq!(
        order_book.arg,
        ChannelArgs {
            channel: "books".to_string(),
            inst_id: "BTC-USD-SWAP".to_string(),
        }
    );
    assert_eq!(order_book.data.len(), 1);
    assert_eq!(order_book.data.first().expect("asks").asks.len(), 400);
    assert_eq!(order_book.data.first().expect("bids").bids.len(), 400);
    assert_eq!(order_book.action, OrderBookAction::Snapshot);
    Ok(())
}

#[tokio::test]
async fn publishes_to_redis() -> anyhow::Result<()> {
    let (tick_send, mut tick_recv) =
        memory::channel(chrono::Duration::from_std(std::time::Duration::from_secs(2)).unwrap());

    let _ = tokio::spawn(async move {
        let _res = okex_price::run(PriceFeedConfig::default(), tick_send).await;
    });

    let received_tick = tick_recv.next().await.expect("expected price tick");

    let payload = &load_fixture()?.payloads[0];
    assert_eq!(received_tick.payload.exchange, payload.exchange);
    assert_eq!(received_tick.payload.instrument_id, payload.instrument_id);

    Ok(())
}
