use futures::StreamExt;
use kollider_price::config::KolliderPriceFeedConfig;
use std::fs;
use url::Url;

use shared::{payload::*, pubsub::*};

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
#[ignore = "currently kolider testnet is flaky"]
async fn publishes_to_redis() -> anyhow::Result<()> {
    let redis_host = std::env::var("REDIS_HOST").unwrap_or("localhost".to_string());
    let pubsub_config = PubSubConfig {
        host: Some(redis_host),
        ..PubSubConfig::default()
    };
    let subscriber = Subscriber::new(pubsub_config.clone()).await?;

    let _ = tokio::spawn(async move {
        let config = KolliderPriceFeedConfig {
            url: Url::parse("wss://testnet.kollider.xyz/v1/ws/").unwrap(),
        };
        let _ = kollider_price::run(config, pubsub_config).await;
    });

    let mut stream = subscriber
        .subscribe::<KolliderBtcUsdSwapPricePayload>()
        .await?;
    let received = stream.next().await.expect("expected price tick");

    let payload = &load_fixture()?.payloads[0];
    assert_eq!(received.payload.exchange, payload.exchange);
    assert_eq!(received.payload.instrument_id, payload.instrument_id);

    Ok(())
}
