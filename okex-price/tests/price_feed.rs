use chrono::Duration;
use futures::StreamExt;
use okex_price::{price_feed::ChannelArgs, subscribe_btc_usd_swap, PriceFeedConfig};
use std::fs;
use url::Url;

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
async fn subscribes_to_okex() -> anyhow::Result<()> {
    let config = PriceFeedConfig {
        url: Url::parse("wss://ws.okx.com:8443/ws/v5/public").unwrap(),
    };
    let mut received = subscribe_btc_usd_swap(config)
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
async fn publishes_to_redis() -> anyhow::Result<()> {
    let redis_host = std::env::var("REDIS_HOST").unwrap_or("localhost".to_string());
    let pubsub_config = PubSubConfig {
        host: Some(redis_host),
        ..PubSubConfig::default()
    };
    let subscriber = Subscriber::new(pubsub_config.clone()).await?;

    let _ = tokio::spawn(async move {
        let _ = okex_price::run(PriceFeedConfig::default(), pubsub_config).await;
    });

    let mut stream = subscriber.subscribe::<OkexBtcUsdSwapPricePayload>().await?;
    let received = stream.next().await.expect("expected price tick");

    let payload = &load_fixture()?.payloads[0];
    assert_eq!(received.payload.exchange, payload.exchange);
    assert_eq!(received.payload.instrument_id, payload.instrument_id);

    Ok(())
}
