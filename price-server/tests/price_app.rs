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
        .get_cents_from_sats_for_immediate_buy(Sats::from_major(100_000_000))
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
        .get_cents_from_sats_for_immediate_buy(Sats::from_major(1100))
        .await?;
    assert_eq!(cents, UsdCents::from_major(315));
    let cents = app
        .get_cents_from_sats_for_immediate_buy(Sats::from_major(1))
        .await?;
    assert_eq!(cents, UsdCents::from_major(0));

    let cents = app
        .get_cents_from_sats_for_immediate_sell(Sats::from_major(1100))
        .await?;
    assert_eq!(cents, UsdCents::from_major(323));
    let cents = app
        .get_cents_from_sats_for_immediate_sell(Sats::from_major(1))
        .await?;
    assert_eq!(cents, UsdCents::from_major(1));

    let cents = app
        .get_cents_from_sats_for_future_buy(Sats::from_major(1100))
        .await?;
    assert_eq!(cents, UsdCents::from_major(286));
    let cents = app
        .get_cents_from_sats_for_future_buy(Sats::from_major(1))
        .await?;
    assert_eq!(cents, UsdCents::from_major(0));

    let future_buy = app
        .get_cents_from_sats_for_future_sell(Sats::from_major(1100))
        .await?;
    assert_eq!(future_buy, UsdCents::from_major(352));
    let future_buy = app
        .get_cents_from_sats_for_future_sell(Sats::from_major(1))
        .await?;
    assert_eq!(future_buy, UsdCents::from_major(1));

    let sats = app
        .get_sats_from_cents_for_immediate_buy(UsdCents::from_major(1000))
        .await?;
    assert_eq!(sats, Sats::from_major(3370));

    let sats = app
        .get_sats_from_cents_for_immediate_sell(UsdCents::from_major(1000))
        .await?;
    assert_eq!(sats, Sats::from_major(3422));
    let sats = app
        .get_sats_from_cents_for_immediate_sell(UsdCents::from_major(1))
        .await?;
    assert_eq!(sats, Sats::from_major(9));

    let sats = app
        .get_sats_from_cents_for_future_buy(UsdCents::from_major(1000))
        .await?;
    assert_eq!(sats, Sats::from_major(3670));
    let sats = app
        .get_sats_from_cents_for_future_buy(UsdCents::from_major(1))
        .await?;
    assert_eq!(sats, Sats::from_major(4));

    let sats = app
        .get_sats_from_cents_for_future_sell(UsdCents::from_major(1000))
        .await?;
    assert_eq!(sats, Sats::from_major(3110));
    let sats = app
        .get_sats_from_cents_for_future_sell(UsdCents::from_major(1))
        .await?;
    assert_eq!(sats, Sats::from_major(8));

    let ratio = app.get_cents_per_sat_exchange_mid_rate().await?;
    assert_eq!(ratio, 0.06);

    Ok(())
}
