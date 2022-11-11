#![allow(clippy::or_fun_call)]

use futures::stream::StreamExt;
use rust_decimal_macros::dec;
use std::{fs, time::Duration};

use price_server::{app::*, *};
use shared::{payload::*, pubsub::*, time::*};

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

    let (_, recv) = futures::channel::mpsc::unbounded();
    let app = PriceApp::run(
        recv,
        PriceServerHealthCheckConfig::default(),
        FeeCalculatorConfig {
            base_fee_rate: dec!(0.001),
            immediate_fee_rate: dec!(0.01),
            delayed_fee_rate: dec!(0.1),
        },
        config,
        ExchangePriceCacheConfig {
            stale_after: Duration::from_secs(30),
        },
    )
    .await?;

    let err = app
        .get_cents_from_sats_for_immediate_buy(Sats::from_major(100_000_000))
        .await;
    if let Err(PriceAppError::ExchangePriceCacheError(ExchangePriceCacheError::NoPriceAvailable)) =
        err
    {
        assert!(true)
    } else {
        panic!()
    }

    let mut payloads = load_fixture()?.payloads.into_iter();
    let mut payload = payloads.next().unwrap();
    publisher
        .publish(OkexBtcUsdSwapPricePayload(payload.clone()))
        .await?;
    stream.next().await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let err = app
        .get_cents_from_sats_for_immediate_buy(Sats::from_major(100_000_000))
        .await;
    if let Err(PriceAppError::ExchangePriceCacheError(ExchangePriceCacheError::StalePrice(_))) = err
    {
        assert!(true)
    } else {
        panic!()
    }

    payload.timestamp = TimeStamp::now();
    publisher
        .publish(OkexBtcUsdSwapPricePayload(payload))
        .await?;
    stream.next().await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let cents = app
        .get_cents_from_sats_for_immediate_buy(Sats::from_major(100_000_000))
        .await?;
    assert_eq!(cents, UsdCents::from_major(98900));
    let cents = app
        .get_cents_from_sats_for_immediate_buy(Sats::from_major(1))
        .await?;
    assert_eq!(cents, UsdCents::from_major(0));

    let cents = app
        .get_cents_from_sats_for_immediate_sell(Sats::from_major(100_000_000))
        .await?;
    assert_eq!(cents, UsdCents::from_major(1011000));
    let cents = app
        .get_cents_from_sats_for_immediate_sell(Sats::from_major(1))
        .await?;
    assert_eq!(cents, UsdCents::from_major(1));

    let cents = app
        .get_cents_from_sats_for_future_buy(Sats::from_major(100_000_000))
        .await?;
    assert_eq!(cents, UsdCents::from_major(89900));
    let cents = app
        .get_cents_from_sats_for_future_buy(Sats::from_major(1))
        .await?;
    assert_eq!(cents, UsdCents::from_major(0));

    let future_buy = app
        .get_cents_from_sats_for_future_sell(Sats::from_major(100_000_000))
        .await?;
    assert_eq!(future_buy, UsdCents::from_major(1101000));
    let future_buy = app
        .get_cents_from_sats_for_future_sell(Sats::from_major(1))
        .await?;
    assert_eq!(future_buy, UsdCents::from_major(1));

    let sats = app
        .get_sats_from_cents_for_immediate_buy(UsdCents::from_major(1000000))
        .await?;
    assert_eq!(sats, Sats::from_major(1011000000));

    let sats = app
        .get_sats_from_cents_for_immediate_sell(UsdCents::from_major(1000000))
        .await?;
    assert_eq!(sats, Sats::from_major(98900000));
    let sats = app
        .get_sats_from_cents_for_immediate_sell(UsdCents::from_major(1))
        .await?;
    assert_eq!(sats, Sats::from_major(98));

    let sats = app
        .get_sats_from_cents_for_future_buy(UsdCents::from_major(1000000))
        .await?;
    assert_eq!(sats, Sats::from_major(1101000000));
    let sats = app
        .get_sats_from_cents_for_future_buy(UsdCents::from_major(1))
        .await?;
    assert_eq!(sats, Sats::from_major(1101));

    let sats = app
        .get_sats_from_cents_for_future_sell(UsdCents::from_major(1000000))
        .await?;
    assert_eq!(sats, Sats::from_major(89900000));
    let sats = app
        .get_sats_from_cents_for_future_sell(UsdCents::from_major(1))
        .await?;
    assert_eq!(sats, Sats::from_major(89));

    let ratio = app.get_cents_per_sat_exchange_mid_rate().await?;
    assert_eq!(ratio, 0.0055);

    Ok(())
}
