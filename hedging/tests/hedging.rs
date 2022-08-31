use futures::stream::StreamExt;
use std::fs;

use shared::{payload::*, pubsub::*, time::*};

use hedging::HedgingApp;

#[derive(serde::Deserialize)]
struct Fixture {
    payloads: Vec<SynthUsdExposurePayload>,
}

fn load_fixture() -> anyhow::Result<Fixture> {
    let contents =
        fs::read_to_string("./tests/fixtures/hedging.json").expect("Couldn't load fixtures");
    Ok(serde_json::from_str(&contents)?)
}

#[tokio::test]
async fn hedging() -> anyhow::Result<()> {
    let redis_host = std::env::var("REDIS_HOST").unwrap_or("localhost".to_string());
    let config = PubSubConfig {
        host: Some(redis_host),
        ..PubSubConfig::default()
    };
    let publisher = Publisher::new(config.clone()).await?;
    let subscriber = Subscriber::new(config.clone()).await?;
    let mut stream = subscriber.subscribe::<SynthUsdExposurePayload>().await?;

    let mut payloads = load_fixture()?.payloads.into_iter();
    let payload = payloads.next().unwrap();
    publisher.publish(payload).await?;

    let app = HedgingApp::run(config).await?;

    let msg = stream.next().await;

    assert!(msg.is_some());

    Ok(())
}
