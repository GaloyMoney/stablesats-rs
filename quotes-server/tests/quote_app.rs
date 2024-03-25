use chrono::Duration;
use rust_decimal_macros::dec;
use serial_test::file_serial;

use quotes_server::error::QuotesAppError;
use quotes_server::{
    app::*, cache::OrderBookCacheError, ExchangePriceCacheError, QuotesExchangePriceCacheConfig,
    QuotesFeeCalculatorConfig,
};

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
#[file_serial]
async fn quotes_app() -> anyhow::Result<()> {
    let (tick_send, tick_recv) =
        memory::channel(chrono::Duration::from_std(std::time::Duration::from_secs(2)).unwrap());
    let publisher = tick_send.clone();
    let mut subscriber = tick_recv.resubscribe();

    let (_, recv) = futures::channel::mpsc::unbounded();
    let ex_cfgs = ExchangeWeights {
        okex: Some(dec!(1.0)),
    };

    let base_fee_rate = dec!(0.001);
    let immediate_fee_rate = dec!(0.01);
    let delayed_fee_rate = dec!(0.1);

    let pg_host = std::env::var("PG_HOST").unwrap_or_else(|_| "localhost".into());
    let pg_con = format!("postgres://user:password@{}:5432/pg", pg_host);
    let pool = sqlx::PgPool::connect(&pg_con).await?;

    let ledger = ledger::Ledger::init(&pool).await?;
    let app = QuotesApp::run(
        pool,
        recv,
        QuotesServerHealthCheckConfig::default(),
        QuotesFeeCalculatorConfig {
            base_fee_rate,
            immediate_fee_rate,
            delayed_fee_rate,
        },
        tick_recv,
        QuotesExchangePriceCacheConfig::default(),
        ex_cfgs,
        QuotesConfig {
            expiration_interval: Duration::try_seconds(2).expect("valid duration"),
        },
        ledger,
    )
    .await?;

    let err = app
        .quote_cents_from_sats_for_buy(dec!(100_000_000), true)
        .await;
    if let Err(QuotesAppError::ExchangePriceCacheError(ExchangePriceCacheError::OrderBookCache(
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

    payload.timestamp = TimeStamp::now();
    publisher
        .publish(PriceStreamPayload::OkexBtcUsdSwapOrderBookPayload(payload))
        .await?;
    subscriber.next().await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let quote = app
        .quote_cents_from_sats_for_buy(dec!(100_000_000), false)
        .await;
    assert!(quote.is_ok());
    let accepted = app.accept_quote(quote.unwrap().id).await;
    assert!(accepted.is_ok());

    let quote = app
        .quote_cents_from_sats_for_buy(dec!(100_000_000), true)
        .await;
    assert!(quote.is_ok());
    assert!(quote.unwrap().is_accepted());

    Ok(())
}
