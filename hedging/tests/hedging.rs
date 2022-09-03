use futures::stream::StreamExt;
use rust_decimal_macros::dec;
use serial_test::serial;

use std::{env, fs};

use okex_client::*;
use shared::{payload::*, pubsub::*};

use hedging::*;

#[derive(serde::Deserialize)]
struct Fixture {
    payloads: Vec<SynthUsdLiabilityPayload>,
}

fn load_fixture() -> anyhow::Result<Fixture> {
    let contents =
        fs::read_to_string("./tests/fixtures/hedging.json").expect("Couldn't load fixtures");
    Ok(serde_json::from_str(&contents)?)
}

fn okex_client_config() -> OkexClientConfig {
    let api_key = env::var("OKEX_API_KEY").expect("OKEX_API_KEY not set");
    let passphrase = env::var("OKEX_PASSPHRASE").expect("OKEX_PASS_PHRASE not set");
    let secret_key = env::var("OKEX_SECRET_KEY").expect("OKEX_SECRET_KEY not set");
    OkexClientConfig {
        api_key,
        passphrase,
        secret_key,
        simulated: true,
    }
}

#[tokio::test]
#[serial]
async fn hedging() -> anyhow::Result<()> {
    let redis_host = std::env::var("REDIS_HOST").unwrap_or("localhost".to_string());
    let pubsub_config = PubSubConfig {
        host: Some(redis_host),
        ..PubSubConfig::default()
    };
    let user_trades_pg_host = std::env::var("HEDGING_PG_HOST").unwrap_or("localhost".to_string());
    let user_trades_pg_port = std::env::var("HEDGING_PG_PORT").unwrap_or("5433".to_string());
    let pg_con = format!(
        "postgres://stablesats:stablesats@{user_trades_pg_host}:{user_trades_pg_port}/stablesats-hedging"
    );

    let publisher = Publisher::new(pubsub_config.clone()).await?;
    let subscriber = Subscriber::new(pubsub_config.clone()).await?;
    let mut stream = subscriber.subscribe::<SynthUsdLiabilityPayload>().await?;

    let _app = HedgingApp::run(
        HedgingAppConfig {
            pg_con,
            migrate_on_start: true,
        },
        okex_client_config(),
        pubsub_config,
    )
    .await?;

    let mut payloads = load_fixture()?.payloads.into_iter();
    publisher.publish(payloads.next().unwrap()).await?;
    let _ = stream.next().await;

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let okex = OkexClient::new(okex_client_config()).await?;
    let mut passed = false;
    let expected = dec!(0);
    for _ in 0..3 {
        let pos = okex.get_position_in_signed_usd().await?.value;
        passed = pos == expected;
        if passed {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    assert!(passed);

    publisher.publish(payloads.next().unwrap()).await?;
    let _ = stream.next().await;
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    let upper_bound = dec!(-950);
    let lower_bound = dec!(-1050);
    passed = false;
    for _ in 0..3 {
        let pos = okex.get_position_in_signed_usd().await?.value;
        if pos < upper_bound && pos > lower_bound {
            passed = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    assert!(passed);

    publisher.publish(payloads.next().unwrap()).await?;
    let _ = stream.next().await;
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    passed = false;
    let expected = dec!(0);
    for _ in 0..3 {
        let pos = okex.get_position_in_signed_usd().await?.value;
        passed = pos == expected;
        if passed {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    assert!(passed);

    Ok(())
}
