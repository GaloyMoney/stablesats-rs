use futures::stream::StreamExt;
use rust_decimal_macros::dec;
use std::fs;

use price_server::{app::*, *};
use shared::{payload::*, pubsub::*, time::*};

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
async fn price_app_with_real_data() -> anyhow::Result<()> {
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

    let err = app
        .get_cents_from_sats_for_immediate_buy(Sats::from_major(100_000_000))
        .await;
    if let Err(PriceAppError::SnapshotCacheError(SnapshotCacheError::NoSnapshotAvailable)) = err {
        assert!(true)
    } else {
        assert!(false)
    }

    let mut payload = load_fixture("real")?.payload;

    publisher
        .publish(OkexBtcUsdSwapOrderBookPayload(payload.clone()))
        .await?;
    stream.next().await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let err = app
        .get_cents_from_sats_for_immediate_buy(Sats::from_major(100_000_000))
        .await;
    if let Err(PriceAppError::SnapshotCacheError(SnapshotCacheError::OutdatedSnapshot(_))) = err {
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
        .get_cents_from_sats_for_immediate_buy(Sats::from_major(100_000_000))
        .await?;
    assert_eq!(cents, UsdCents::from_major(1997809));
    let cents = app
        .get_cents_from_sats_for_immediate_buy(Sats::from_major(1))
        .await?;
    assert_eq!(cents, UsdCents::from_major(0));

    let cents = app
        .get_cents_from_sats_for_immediate_sell(Sats::from_major(100_000_000))
        .await?;
    assert_eq!(cents, UsdCents::from_major(2080253));
    let cents = app
        .get_cents_from_sats_for_immediate_sell(Sats::from_major(1))
        .await?;
    assert_eq!(cents, UsdCents::from_major(1));

    let cents = app
        .get_cents_from_sats_for_future_buy(Sats::from_major(100_000_000))
        .await?;
    assert_eq!(cents, UsdCents::from_major(1816006));
    let cents = app
        .get_cents_from_sats_for_future_buy(Sats::from_major(1))
        .await?;
    assert_eq!(cents, UsdCents::from_major(0));

    let future_buy = app
        .get_cents_from_sats_for_future_sell(Sats::from_major(100_000_000))
        .await?;
    assert_eq!(future_buy, UsdCents::from_major(2265439));
    let future_buy = app
        .get_cents_from_sats_for_future_sell(Sats::from_major(1))
        .await?;
    assert_eq!(future_buy, UsdCents::from_major(1));

    let sats = app
        .get_sats_from_cents_for_immediate_buy(UsdCents::from_major(1000))
        .await?;
    assert_eq!(sats, Sats::from_major(50049));

    let sats = app
        .get_sats_from_cents_for_immediate_sell(UsdCents::from_major(1000))
        .await?;
    assert_eq!(sats, Sats::from_major(48065));
    let sats = app
        .get_sats_from_cents_for_immediate_sell(UsdCents::from_major(1))
        .await?;
    assert_eq!(sats, Sats::from_major(48));

    let sats = app
        .get_sats_from_cents_for_future_buy(UsdCents::from_major(1000))
        .await?;
    assert_eq!(sats, Sats::from_major(54505));
    let sats = app
        .get_sats_from_cents_for_future_buy(UsdCents::from_major(1))
        .await?;
    assert_eq!(sats, Sats::from_major(55));

    let sats = app
        .get_sats_from_cents_for_future_sell(UsdCents::from_major(1000))
        .await?;
    assert_eq!(sats, Sats::from_major(43691));
    let sats = app
        .get_sats_from_cents_for_future_sell(UsdCents::from_major(1))
        .await?;
    assert_eq!(sats, Sats::from_major(43));

    let ratio = app.get_cents_per_sat_exchange_mid_rate().await?;
    assert_eq!(ratio, 0.020388242306794459);

    Ok(())
}

#[tokio::test]
async fn price_app_with_contrived_data() -> anyhow::Result<()> {
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

    let err = app
        .get_cents_from_sats_for_immediate_buy(Sats::from_major(100_000_000))
        .await;
    if let Err(PriceAppError::SnapshotCacheError(SnapshotCacheError::NoSnapshotAvailable)) = err {
        assert!(true)
    } else {
        assert!(false)
    }

    let mut payload = load_fixture("contrived")?.payload;

    publisher
        .publish(OkexBtcUsdSwapOrderBookPayload(payload.clone()))
        .await?;
    stream.next().await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let err = app
        .get_cents_from_sats_for_immediate_buy(Sats::from_major(100_000_000))
        .await;
    if let Err(PriceAppError::SnapshotCacheError(SnapshotCacheError::OutdatedSnapshot(_))) = err {
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
        .get_cents_from_sats_for_immediate_buy(Sats::from_major(100_000_000))
        .await?;
    assert_eq!(cents, UsdCents::from_major(989000));
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
    assert_eq!(cents, UsdCents::from_major(899000));
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
    assert_eq!(sats, Sats::from_major(101100000));

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
    assert_eq!(sats, Sats::from_major(110100000));
    let sats = app
        .get_sats_from_cents_for_future_buy(UsdCents::from_major(1))
        .await?;
    assert_eq!(sats, Sats::from_major(111));

    let sats = app
        .get_sats_from_cents_for_future_sell(UsdCents::from_major(1000000))
        .await?;
    assert_eq!(sats, Sats::from_major(89900000));
    let sats = app
        .get_sats_from_cents_for_future_sell(UsdCents::from_major(1))
        .await?;
    assert_eq!(sats, Sats::from_major(89));

    let ratio = app.get_cents_per_sat_exchange_mid_rate().await?;
    assert_eq!(ratio, 0.01);

    Ok(())
}
