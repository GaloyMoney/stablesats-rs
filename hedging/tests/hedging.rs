use futures::stream::StreamExt;
use std::fs;

use shared::{payload::*, pubsub::*, time::*};

use hedging::*;

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
    let pubsub_config = PubSubConfig {
        host: Some(redis_host),
        ..PubSubConfig::default()
    };
    let subscriber = Subscriber::new(pubsub_config.clone()).await?;
    let user_trades_pg_host = std::env::var("HEDGING_PG_HOST").unwrap_or("localhost".to_string());
    let user_trades_pg_port = std::env::var("HEDGING_PG_PORT").unwrap_or("5433".to_string());
    let pg_con = format!(
        "postgres://stablesats:stablesats@{user_trades_pg_host}:{user_trades_pg_port}/stablesats-hedging"
    );

    let publisher = Publisher::new(pubsub_config.clone()).await?;
    let subscriber = Subscriber::new(pubsub_config.clone()).await?;
    let mut stream = subscriber.subscribe::<SynthUsdExposurePayload>().await?;

    let mut payloads = load_fixture()?.payloads.into_iter();
    let payload = payloads.next().unwrap();
    publisher.publish(payload).await?;

    let app = HedgingApp::run(
        HedgingAppConfig {
            pg_con: pg_con,
            migrate_on_start: false,
        },
        pubsub_config,
    )
    .await?;

    let msg = stream.next().await;

    assert!(msg.is_some());

    // tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    // assert!(false);
    Ok(())
}
