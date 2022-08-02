use price_server::app::*;
use shared::{payload::*, pubsub::*, time::*};
use std::fs;

#[derive(serde::Deserialize)]
struct Fixture {
    payloads: Vec<PriceMessagePayload>,
}

fn load_fixture() -> anyhow::Result<Fixture> {
    let contents =
        fs::read_to_string("./tests/fixtures/price_app.json").expect("Couldn't load fixtures");
    Ok(serde_json::from_str(&contents)?)
}

#[tokio::test]
async fn price_app_test() -> anyhow::Result<()> {
    let redis_host = std::env::var("REDIS_HOST").unwrap_or("localhost".to_string());
    let config = PubSubConfig {
        host: Some(redis_host),
    };
    let publisher = Publisher::new(config.clone()).await?;

    let _app = PriceApp::run(config).await?;

    let mut payloads = load_fixture()?.payloads.into_iter();
    let mut payload = payloads.next().unwrap();
    payload.timestamp = TimeStamp::now();
    publisher
        .publish(OkexBtcUsdSwapPricePayload(payload))
        .await?;

    // let cents = app
    //     .get_cents_from_sats_for_immediate_buy(Sats::from_major(1))
    //     .await?;
    Ok(())
}
