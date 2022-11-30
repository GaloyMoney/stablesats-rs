use futures::stream::StreamExt;
use rust_decimal_macros::dec;
use std::fs;

use price_server::{app::*, ExchangePriceCacheError};
use shared::{
    exchanges_config::{ExchangeConfig, ExchangeConfigAll, OkexConfig},
    payload::*,
    pubsub::*,
    time::*,
};

#[derive(serde::Deserialize)]
struct Fixture {
    payload: OrderBookPayload,
}

fn load_fixture(dataname: &str) -> anyhow::Result<Fixture> {
    let contents = fs::read_to_string(format!(
        "./tests/fixtures/order-book-payload-{}.json",
        dataname
    ))
    .expect("Couldn't load fixtures");
    Ok(serde_json::from_str(&contents)?)
}

#[tokio::test]
async fn price_app_with_order_book_cache() -> anyhow::Result<()> {
    let redis_host = std::env::var("REDIS_HOST").unwrap_or("localhost".to_string());
    let config = PubSubConfig {
        host: Some(redis_host),
        ..PubSubConfig::default()
    };
    let publisher = Publisher::new(config.clone()).await?;
    let subscriber = Subscriber::new(config.clone()).await?;
    let mut stream = subscriber
        .subscribe::<OkexBtcUsdSwapOrderBookPayload>()
        .await?;

    let (_, recv) = futures::channel::mpsc::unbounded();

    let okex = ExchangeConfig {
        weight: dec!(1.0),
        config: OkexConfig {
            api_key: "okex api".to_string(),
            passphrase: "passphrase".to_string(),
            secret_key: "secret key".to_string(),
            simulated: false,
        },
    };

    let ex_cfgs = ExchangeConfigAll {
        okex: Some(okex),
        kollider: None,
    };

    let app = PriceApp::run(
        recv,
        FeeCalculatorConfig {
            base_fee_rate: dec!(0.001),
            immediate_fee_rate: dec!(0.01),
            delayed_fee_rate: dec!(0.1),
        },
        config,
        ex_cfgs,
    )
    .await?;

    let err = app
        .get_cents_from_sats_for_immediate_buy(Sats::from_major(100_000_000))
        .await;
    if let Err(PriceAppError::OrderBookCacheError(OrderBookCacheError::NoSnapshotAvailable)) = err {
        assert!(true)
    } else {
        assert!(false)
    }

    let mut payloads = load_fixture()?.payloads.into_iter();
    let mut payload = payloads.next().unwrap();

    publisher
        .publish(OkexBtcUsdSwapOrderBookPayload(payload.clone()))
        .await?;
    stream.next().await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let err = app
        .get_cents_from_sats_for_immediate_buy(Sats::from_major(100))
        .await;
    if let Err(PriceAppError::OrderBookCacheError(OrderBookCacheError::OutdatedSnapshot(_))) = err {
        assert!(true)
    } else {
        assert!(false)
    }

    payload.timestamp = TimeStamp::now();
    publisher
        .publish(OkexBtcUsdSwapOrderBookPayload(payload))
        .await?;
    stream.next().await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let cents = app
        .get_cents_from_sats_for_immediate_buy(Sats::from_major(100))
        .await?;
    assert_eq!(cents, UsdCents::from_major(24));
    let cents = app
        .get_cents_from_sats_for_immediate_buy(Sats::from_major(1))
        .await?;
    assert_eq!(cents, UsdCents::from_major(0));

    let cents = app
        .get_cents_from_sats_for_immediate_sell(Sats::from_major(100))
        .await?;
    assert_eq!(cents, UsdCents::from_major(20));
    let cents = app
        .get_cents_from_sats_for_immediate_sell(Sats::from_major(1))
        .await?;
    assert_eq!(cents, UsdCents::from_major(1));

    let cents = app
        .get_cents_from_sats_for_future_buy(Sats::from_major(100))
        .await?;
    assert_eq!(cents, UsdCents::from_major(22));
    let cents = app
        .get_cents_from_sats_for_future_buy(Sats::from_major(1))
        .await?;
    assert_eq!(cents, UsdCents::from_major(0));

    let future_buy = app
        .get_cents_from_sats_for_future_sell(Sats::from_major(100))
        .await?;
    assert_eq!(future_buy, UsdCents::from_major(21));
    let future_buy = app
        .get_cents_from_sats_for_future_sell(Sats::from_major(1))
        .await?;
    assert_eq!(future_buy, UsdCents::from_major(1));

    let sats = app
        .get_sats_from_cents_for_immediate_buy(UsdCents::from_major(10))
        .await?;
    assert_eq!(sats, Sats::from_major(41));

    let sats = app
        .get_sats_from_cents_for_immediate_sell(UsdCents::from_major(10))
        .await?;
    assert_eq!(sats, Sats::from_major(98));
    let sats = app
        .get_sats_from_cents_for_immediate_sell(UsdCents::from_major(1))
        .await?;
    assert_eq!(sats, Sats::from_major(9));

    let sats = app
        .get_sats_from_cents_for_future_buy(UsdCents::from_major(10))
        .await?;
    assert_eq!(sats, Sats::from_major(45));
    let sats = app
        .get_sats_from_cents_for_future_buy(UsdCents::from_major(1))
        .await?;
    assert_eq!(sats, Sats::from_major(5));

    let sats = app
        .get_sats_from_cents_for_future_sell(UsdCents::from_major(10))
        .await?;
    assert_eq!(sats, Sats::from_major(89));
    let sats = app
        .get_sats_from_cents_for_future_sell(UsdCents::from_major(1))
        .await?;
    assert_eq!(sats, Sats::from_major(8));

    let ratio = app.get_cents_per_sat_exchange_mid_rate().await?;
    assert_eq!(ratio, 0.175);

    Ok(())
}

#[tokio::test]
async fn price_app_round_trip() -> anyhow::Result<()> {
    let redis_host = std::env::var("REDIS_HOST").unwrap_or("localhost".to_string());
    let config = PubSubConfig {
        host: Some(redis_host),
        ..PubSubConfig::default()
    };
    let publisher = Publisher::new(config.clone()).await?;
    let subscriber = Subscriber::new(config.clone()).await?;
    let mut stream = subscriber
        .subscribe::<OkexBtcUsdSwapOrderBookPayload>()
        .await?;

    let (_, recv) = futures::channel::mpsc::unbounded();
    let app = PriceApp::run(
        recv,
        FeeCalculatorConfig {
            base_fee_rate: dec!(0.001),
            immediate_fee_rate: dec!(0.01),
            delayed_fee_rate: dec!(0.1),
        },
        config,
    )
    .await?;

    let mut payload = load_fixture("real")?.payload;
    payload.timestamp = TimeStamp::now();
    publisher
        .publish(OkexBtcUsdSwapOrderBookPayload(payload.clone()))
        .await?;
    stream.next().await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let starting_sats = vec![
        Sats::from_major(1),
        Sats::from_major(10),
        Sats::from_major(100),
        Sats::from_major(1_000),
        Sats::from_major(10_000),
        Sats::from_major(100_000),
    ];

    // Immediate Buy: Sats=>Cents=>Sats
    for (idx, value) in starting_sats.iter().enumerate() {
        let cents = app
            .get_cents_from_sats_for_immediate_buy(value.clone())
            .await?;
        let ending_sats = app.get_sats_from_cents_for_immediate_buy(cents).await?;
        assert!(ending_sats.amount() <= starting_sats[idx].amount());
    }

    // Future Buy: Sats=>Cents=>Sats
    for (idx, value) in starting_sats.iter().enumerate() {
        let cents = app
            .get_cents_from_sats_for_future_buy(value.clone())
            .await?;
        let ending_sats = app.get_sats_from_cents_for_future_buy(cents).await?;
        assert!(ending_sats.amount() <= starting_sats[idx].amount());
    }

    let starting_cents = vec![
        UsdCents::from_major(1),
        UsdCents::from_major(10),
        UsdCents::from_major(100),
        UsdCents::from_major(1_000),
        UsdCents::from_major(10_000),
        UsdCents::from_major(100_000),
    ];

    // Immediate Buy: Cents=>Sats=>Cents
    for (idx, value) in starting_cents.iter().enumerate() {
        let sats = app
            .get_sats_from_cents_for_immediate_buy(value.clone())
            .await?;
        let ending_cents = app.get_cents_from_sats_for_immediate_buy(sats).await?;
        assert!(ending_cents.amount() <= starting_cents[idx].amount());
    }

    // Future Buy: Cents=>Sats=>Cents
    for (idx, value) in starting_cents.iter().enumerate() {
        let sats = app
            .get_sats_from_cents_for_future_buy(value.clone())
            .await?;
        let ending_cents = app.get_cents_from_sats_for_future_buy(sats).await?;
        assert!(ending_cents.amount() <= starting_cents[idx].amount());
    }

    Ok(())
}
