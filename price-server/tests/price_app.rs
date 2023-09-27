use rust_decimal_macros::dec;

use price_server::{app::*, ExchangePriceCacheConfig, OrderBookCacheError};
use shared::{payload::*, pubsub::*, time::*};

fn load_fixture() -> OrderBookPayload {
    OrderBookPayload {
        bids: [(
            PriceRaw::from(dec!(0.001)),
            VolumeInCentsRaw::from(dec!(100_000_000)),
        )]
        .into_iter()
        .collect(),
        asks: [(
            PriceRaw::from(dec!(0.01)),
            VolumeInCentsRaw::from(dec!(100_000_000)),
        )]
        .into_iter()
        .collect(),
        timestamp: TimeStamp::from(10000000),
        exchange: "okex".into(),
    }
}

#[tokio::test]
async fn price_app() -> anyhow::Result<()> {
    let (tick_send, tick_recv) =
        memory::channel(chrono::Duration::from_std(std::time::Duration::from_secs(2)).unwrap());
    let publisher = tick_send.clone();
    let mut subscriber = tick_recv.resubscribe();

    let (_, recv) = futures::channel::mpsc::unbounded();

    let ex_cfgs = ExchangeWeights {
        okex: Some(dec!(1.0)),
        bitfinex: None,
    };

    let base_fee_rate = dec!(0.001);
    let immediate_fee_rate = dec!(0.01);
    let delayed_fee_rate = dec!(0.1);
    let app = PriceApp::run(
        recv,
        PriceServerHealthCheckConfig::default(),
        FeeCalculatorConfig {
            base_fee_rate,
            immediate_fee_rate,
            delayed_fee_rate,
        },
        tick_recv,
        ExchangePriceCacheConfig::default(),
        ex_cfgs,
    )
    .await?;

    let err = app
        .get_cents_from_sats_for_immediate_buy(Sats::from_major(100_000_000))
        .await;
    if let Err(PriceAppError::ExchangePriceCacheError(ExchangePriceCacheError::OrderBookCache(
        OrderBookCacheError::NoSnapshotAvailable,
    ))) = err
    {
        assert!(true)
    } else {
        assert!(false)
    }

    let mut payload = load_fixture();
    tick_send
        .publish(PriceStreamPayload::OkexBtcUsdSwapOrderBookPayload(
            payload.clone(),
        ))
        .await?;
    subscriber.next().await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let err = app
        .get_cents_from_sats_for_immediate_buy(Sats::from_major(100_000_000))
        .await;
    if let Err(PriceAppError::ExchangePriceCacheError(ExchangePriceCacheError::OrderBookCache(
        OrderBookCacheError::OutdatedSnapshot(_),
    ))) = err
    {
        assert!(true)
    } else {
        assert!(false)
    }

    payload.timestamp = TimeStamp::now();
    publisher
        .publish(PriceStreamPayload::OkexBtcUsdSwapOrderBookPayload(payload))
        .await?;
    subscriber.next().await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let cents = app
        .get_cents_from_sats_for_immediate_buy(Sats::from_major(100_000_000))
        .await?;
    assert_eq!(cents, UsdCents::from_major(98_900));

    let cents = app
        .get_cents_from_sats_for_immediate_buy(Sats::from_major(1))
        .await?;
    assert_eq!(cents, UsdCents::from_major(0));

    let cents = app
        .get_cents_from_sats_for_immediate_sell(Sats::from_major(100_000_000))
        .await?;
    assert_eq!(cents, UsdCents::from_major(1_011_000));

    let cents = app
        .get_cents_from_sats_for_immediate_sell(Sats::from_major(1))
        .await?;
    assert_eq!(cents, UsdCents::from_major(1));

    let cents = app
        .get_cents_from_sats_for_future_buy(Sats::from_major(100_000_000))
        .await?;
    assert_eq!(cents, UsdCents::from_major(89_900));

    let cents = app
        .get_cents_from_sats_for_future_buy(Sats::from_major(1))
        .await?;
    assert_eq!(cents, UsdCents::from_major(0));

    let cents = app
        .get_cents_from_sats_for_future_sell(Sats::from_major(100_000_000))
        .await?;
    assert_eq!(cents, UsdCents::from_major(1_101_000));
    let cents = app
        .get_cents_from_sats_for_future_sell(Sats::from_major(1))
        .await?;
    assert_eq!(cents, UsdCents::from_major(1));

    let sats = app
        .get_sats_from_cents_for_immediate_buy(UsdCents::from_major(1000000))
        .await?;
    assert_eq!(sats, Sats::from_major(1_011_000_000));

    let sats = app
        .get_sats_from_cents_for_immediate_sell(UsdCents::from_major(1000000))
        .await?;
    assert_eq!(sats, Sats::from_major(98_900_000));
    let sats = app
        .get_sats_from_cents_for_immediate_sell(UsdCents::from_major(1))
        .await?;
    assert_eq!(sats, Sats::from_major(98));

    let sats = app
        .get_sats_from_cents_for_future_buy(UsdCents::from_major(1000000))
        .await?;
    assert_eq!(sats, Sats::from_major(1_101_000_000));
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
