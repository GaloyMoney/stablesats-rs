use futures::stream::StreamExt;
use rust_decimal::prelude::*;
use std::fs;

use price_server::app::*;
use shared::{currency::*, payload::*, pubsub::*, time::*};

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
async fn price_app() -> anyhow::Result<()> {
    let redis_host = std::env::var("REDIS_HOST").unwrap_or("localhost".to_string());
    let config = PubSubConfig {
        host: Some(redis_host),
        ..PubSubConfig::default()
    };
    let publisher = Publisher::new(config.clone()).await?;
    let subscriber = Subscriber::new(config.clone()).await?;
    let mut stream = subscriber.subscribe::<OkexBtcUsdSwapPricePayload>().await?;

    let app = PriceApp::run(
        FeeCalculatorConfig {
            base_fee_rate: Decimal::from_str("0.001")?,
            immediate_fee_rate: Decimal::from_str("0.01")?,
            delayed_fee_rate: Decimal::from_str("0.1")?,
        },
        config,
    )
    .await?;

    let err = app
        .get_cents_from_sats_for_immediate_buy(Sats::from_major(100_000_000))
        .await;
    assert_eq!(
        format!("{}", err.unwrap_err()),
        "PriceAppError: No price data available"
    );

    let mut payloads = load_fixture()?.payloads.into_iter();
    let mut payload = payloads.next().unwrap();
    publisher
        .publish(OkexBtcUsdSwapPricePayload(payload.clone()))
        .await?;
    stream.next().await;
    let err = app
        .get_cents_from_sats_for_immediate_buy(Sats::from_major(100_000_000))
        .await;
    assert_eq!(
        format!("{}", err.unwrap_err()),
        "PriceAppError: StalePrice: last update was at 1"
    );

    payload.timestamp = TimeStamp::now();
    publisher
        .publish(OkexBtcUsdSwapPricePayload(payload))
        .await?;
    stream.next().await;

    let cents = app
        .get_cents_from_sats_for_immediate_buy(Sats::from_major(100_000_000))
        .await?;
    assert_eq!(cents, UsdCents::from_major(98900));

    let cents = app
        .get_cents_from_sats_for_immediate_sell(Sats::from_major(100_000_000))
        .await?;
    assert_eq!(cents, UsdCents::from_major(1011000));

    let cents = app
        .get_cents_from_sats_for_future_buy(Sats::from_major(100_000_000))
        .await?;
    assert_eq!(cents, UsdCents::from_major(89900));

    let future_buy = app
        .get_cents_from_sats_for_future_sell(Sats::from_major(100_000_000))
        .await?;
    assert_eq!(future_buy, UsdCents::from_major(1101000));

    let sats = app
        .get_sats_from_cents_for_immediate_buy(UsdCents::from_major(1000000))
        .await?;
    assert_eq!(sats, Sats::from_major(989000000));

    let sats = app
        .get_sats_from_cents_for_immediate_sell(UsdCents::from_major(1000000))
        .await?;
    assert_eq!(sats, Sats::from_major(101100000));

    let sats = app
        .get_sats_from_cents_for_future_buy(UsdCents::from_major(1000000))
        .await?;
    assert_eq!(sats, Sats::from_major(899000000));

    let sats = app
        .get_sats_from_cents_for_future_sell(UsdCents::from_major(1000000))
        .await?;
    assert_eq!(sats, Sats::from_major(110100000));

    let ratio = app.get_cents_per_sat_exchange_mid_rate().await?;
    assert_eq!(ratio, 0.0055);

    Ok(())
}
