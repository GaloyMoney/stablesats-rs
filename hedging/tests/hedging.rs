use futures::{stream::StreamExt, Stream};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serial_test::serial;

use std::{env, fs, pin::Pin};

use okex_client::*;
use shared::{payload::*, pubsub::*};

use hedging::*;

#[derive(serde::Deserialize)]
struct Fixture {
    payloads: Vec<SynthUsdLiabilityPayload>,
}

fn load_fixture(path: &str) -> anyhow::Result<Fixture> {
    let contents = fs::read_to_string(path).expect("Couldn't load fixtures");
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

async fn expect_exposure_between(
    stream: &mut Pin<Box<dyn Stream<Item = Envelope<OkexBtcUsdSwapPositionPayload>> + Send>>,
    lower: Decimal,
    upper: Decimal,
) {
    let mut passed = false;
    for _ in 0..=10 {
        let pos = stream.next().await.unwrap().payload.signed_usd_exposure;
        passed = pos < upper && pos > lower;
        if passed {
            break;
        }
    }
    assert!(passed);
}

async fn expect_exposure_below(
    stream: &mut Pin<Box<dyn Stream<Item = Envelope<OkexBtcUsdSwapPositionPayload>> + Send>>,
    expected: Decimal,
) {
    let mut passed = false;
    for _ in 0..=10 {
        let pos = stream.next().await.unwrap().payload.signed_usd_exposure;
        passed = pos < expected;
        if passed {
            break;
        }
    }
    assert!(passed);
}

async fn expect_exposure_equal(
    stream: &mut Pin<Box<dyn Stream<Item = Envelope<OkexBtcUsdSwapPositionPayload>> + Send>>,
    expected: Decimal,
) {
    let mut passed = false;
    for _ in 0..=10 {
        let pos = stream.next().await.unwrap().payload.signed_usd_exposure;
        passed = pos == expected;
        if passed {
            break;
        }
    }
    assert!(passed);
}

#[tokio::test]
#[serial]
async fn scenario_01() -> anyhow::Result<()> {
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
    let mut stream = subscriber
        .subscribe::<OkexBtcUsdSwapPositionPayload>()
        .await?;

    tokio::spawn(async move {
        HedgingApp::run(
            HedgingAppConfig {
                pg_con,
                migrate_on_start: true,
                okex_poll_frequency: std::time::Duration::from_secs(1),
            },
            okex_client_config(),
            pubsub_config,
        )
        .await
    });
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    let expected_exposure = [
        (-1, 1),
        (-101, -99),
        (-201, -199),
        (-101, -99),
        (-201, -199),
        (-101, -99),
        (-1, 1),
        (-1, 1),
        (-101, -99),
        (-101, -99),
        (-1, 1),
        (-1, 1),
    ];
    let mut payloads = load_fixture("./tests/fixtures/scenario_01.json")
        .expect("Couldn't load fixtures")
        .payloads
        .into_iter();
    for (exposure_low, exposure_up) in expected_exposure {
        publisher.publish(payloads.next().unwrap()).await?;
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        expect_exposure_between(
            &mut stream,
            Decimal::from(exposure_low),
            Decimal::from(exposure_up),
        )
        .await;
    }

    Ok(())
}
