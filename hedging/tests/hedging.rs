#![allow(clippy::or_fun_call)]

use futures::{stream::StreamExt, Stream};
use galoy_client::GaloyClientConfig;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serial_test::serial;

use std::{env, fs};

use ledger::*;
use okex_client::*;
use shared::pubsub::*;

use hedging::*;

// #[derive(serde::Deserialize)]
// struct Fixture {
//     payloads: Vec<SynthUsdLiabilityPayload>,
// }

// fn load_fixture(path: &str) -> anyhow::Result<Fixture> {
//     let contents = fs::read_to_string(path).expect("Couldn't load fixtures");
//     Ok(serde_json::from_str(&contents)?)
// }

fn okex_config() -> OkexConfig {
    let api_key = env::var("OKEX_API_KEY").expect("OKEX_API_KEY not set");
    let passphrase = env::var("OKEX_PASSPHRASE").expect("OKEX_PASS_PHRASE not set");
    let secret_key = env::var("OKEX_SECRET_KEY").expect("OKEX_SECRET_KEY not set");
    OkexConfig {
        client: OkexClientConfig {
            api_key,
            passphrase,
            secret_key,
            simulated: true,
        },
        ..Default::default()
    }
}

fn galoy_client_config() -> GaloyClientConfig {
    let api = env::var("GALOY_GRAPHQL_URI").expect("GALOY_GRAPHQL_URI not set");
    let phone_number = env::var("PHONE_NUMBER").expect("PHONE_NUMBER not set");
    let code = env::var("AUTH_CODE").expect("AUTH_CODE not set");

    GaloyClientConfig {
        api,
        phone_number,
        auth_code: code,
    }
}

// async fn expect_exposure_between(
//     mut stream: impl Stream<Item = Envelope<OkexBtcUsdSwapPositionPayload>> + Unpin,
//     lower: Decimal,
//     upper: Decimal,
// ) {
//     let mut passed = false;
//     for _ in 0..=20 {
//         let pos = stream.next().await.unwrap().payload.signed_usd_exposure;
//         passed = pos < upper && pos > lower;
//         if passed {
//             break;
//         }
//     }
//     assert!(passed);
// }

// async fn expect_exposure_below(
//     mut stream: impl Stream<Item = Envelope<OkexBtcUsdSwapPositionPayload>> + Unpin,
//     expected: Decimal,
// ) {
//     let mut passed = false;
//     for _ in 0..=10 {
//         let pos = stream.next().await.unwrap().payload.signed_usd_exposure;
//         passed = pos < expected;
//         if passed {
//             break;
//         }
//         tokio::time::sleep(std::time::Duration::from_millis(200)).await;
//     }
//     assert!(passed);
// }

// async fn expect_exposure_equal(
//     mut stream: impl Stream<Item = Envelope<OkexBtcUsdSwapPositionPayload>> + Unpin,
//     expected: Decimal,
// ) {
//     let mut passed = false;
//     for _ in 0..=20 {
//         let pos = stream.next().await.unwrap().payload.signed_usd_exposure;
//         passed = pos == expected;
//         if passed {
//             break;
//         }
//     }
//     assert!(passed);
// }

// #[tokio::test]
// #[serial]
// #[ignore = "okex is very unstable"]
// async fn hedging() -> anyhow::Result<()> {
//     let (_, tick_recv) = memory::channel(chrono::Duration::from_std(
//         std::time::Duration::from_secs(1),
//     )?);
//     let redis_host = std::env::var("REDIS_HOST").unwrap_or("localhost".to_string());
//     let pubsub_config = PubSubConfig {
//         host: Some(redis_host),
//         ..PubSubConfig::default()
//     };

//     let publisher = Publisher::new(pubsub_config.clone()).await?;
//     let mut subscriber = Subscriber::new(pubsub_config.clone()).await?;
//     let mut stream = subscriber
//         .subscribe::<OkexBtcUsdSwapPositionPayload>()
//         .await?;

//     let mut payloads = load_fixture("./tests/fixtures/hedging.json")
//         .expect("Couldn't load fixtures")
//         .payloads
//         .into_iter();
//     publisher.publish(payloads.next().unwrap()).await?;

//     let okex = OkexClient::new(okex_config().client).await?;
//     expect_exposure_equal(&mut stream, dec!(0)).await;

//     publisher.publish(payloads.next().unwrap()).await?;

//     for idx in 0..=1 {
//         expect_exposure_between(&mut stream, dec!(-21000), dec!(-19000)).await;

//         if idx == 0 {
//             okex.place_order(
//                 ClientOrderId::new(),
//                 OkexOrderSide::Sell,
//                 &BtcUsdSwapContracts::from(5),
//             )
//             .await?;
//             expect_exposure_below(&mut stream, dec!(-50000)).await;
//         }
//     }

//     publisher.publish(payloads.next().unwrap()).await?;

//     expect_exposure_equal(&mut stream, dec!(0)).await;

//     Ok(())
// }

#[tokio::test]
#[serial]
#[ignore]
async fn hedging() -> anyhow::Result<()> {
    let (send, mut receive) = tokio::sync::mpsc::channel(1);
    let (_, tick_recv) = memory::channel(chrono::Duration::from_std(
        std::time::Duration::from_secs(1),
    )?);
    let pg_host = std::env::var("PG_HOST").unwrap_or("localhost".to_string());
    let pg_con = format!("postgres://user:password@{pg_host}:5432/pg",);
    let pool = sqlx::PgPool::connect(&pg_con).await?;
    let ledger = ledger::Ledger::init(&pool).await?;
    let cloned_pool = pool.clone();
    let hedging_send = send.clone();
    tokio::spawn(async move {
        let (_, recv) = futures::channel::mpsc::unbounded();
        let _ = hedging_send.try_send(
            HedgingApp::run(
                cloned_pool,
                recv,
                HedgingAppConfig {
                    ..Default::default()
                },
                okex_config(),
                galoy_client_config(),
                tick_recv.resubscribe(),
            )
            .await
            .expect("HedgingApp failed"),
        );
    });
    let _reason = receive.recv().await.expect("Didn't receive msg");
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    ledger
        .user_buys_usd(
            pool.begin().await?,
            LedgerTxId::new(),
            UserBuysUsdParams {
                satoshi_amount: dec!(1000000),
                usd_cents_amount: dec!(50000),
                meta: UserBuysUsdMeta {
                    timestamp: chrono::Utc::now(),
                    btc_tx_id: "btc_tx_id".to_string(),
                    usd_tx_id: "usd_tx_id".to_string(),
                },
            },
        )
        .await?;
    tokio::time::sleep(std::time::Duration::from_secs(15)).await;
    Ok(())
}
