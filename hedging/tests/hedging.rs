#![allow(clippy::or_fun_call)]

use galoy_client::GaloyClientConfig;
use rust_decimal_macros::dec;
use serial_test::serial;

use std::env;

use ledger::*;
use okex_client::*;
use shared::pubsub::*;

use hedging::*;

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

#[tokio::test]
#[serial]
async fn hedging() -> anyhow::Result<()> {
    let pg_host = std::env::var("PG_HOST").unwrap_or_else(|_| "localhost".into());
    let pg_con = format!("postgres://user:password@{}:5432/pg", pg_host);
    let pool = sqlx::PgPool::connect(&pg_con).await?;
    let ledger = ledger::Ledger::init(&pool).await?;

    let (send, mut receive) = tokio::sync::mpsc::channel(1);
    let (_, tick_recv) = memory::channel(chrono::Duration::from_std(
        std::time::Duration::from_secs(1),
    )?);

    tokio::spawn(async move {
        let (_, recv) = futures::channel::mpsc::unbounded();
        let _ = send.try_send(
            HedgingApp::run(
                pool,
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
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    let pool = sqlx::PgPool::connect(&pg_con).await?;
    ledger
        .user_buys_usd(
            pool.clone().begin().await?,
            LedgerTxId::new(),
            UserBuysUsdParams {
                satoshi_amount: dec!(1000000),
                usd_cents_amount: dec!(50000),
                meta: UserBuysUsdMeta {
                    timestamp: chrono::Utc::now(),
                    btc_tx_id: "btc_tx_id".into(),
                    usd_tx_id: "usd_tx_id".into(),
                },
            },
        )
        .await?;
    let mut event = ledger.usd_okex_position_balance_events().await?;
    let mut passed = false;
    for _ in 0..=20 {
        let user_buy_event = event.recv().await?;
        // checks if a position of $-500 gets opened on the exchange.
        if let ledger::LedgerEventData::BalanceUpdated(data) = user_buy_event.data {
            if (data.settled_cr_balance - data.settled_dr_balance) == dec!(-500) {
                passed = true;
                break;
            }
        } else {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }
    if !passed {
        panic!("Could not open a position on the exchange!");
    }

    ledger
        .user_sells_usd(
            pool.begin().await?,
            LedgerTxId::new(),
            UserSellsUsdParams {
                satoshi_amount: dec!(1000000),
                usd_cents_amount: dec!(50000),
                meta: UserSellsUsdMeta {
                    timestamp: chrono::Utc::now(),
                    btc_tx_id: "btc_tx_id".into(),
                    usd_tx_id: "usd_tx_id".into(),
                },
            },
        )
        .await?;
    let mut passed = false;
    for _ in 0..=20 {
        let user_sell_event = event.recv().await?;
        // checks if the position gets closed on the exchange.
        if let ledger::LedgerEventData::BalanceUpdated(data) = user_sell_event.data {
            if (data.settled_cr_balance - data.settled_dr_balance) == dec!(0) {
                passed = true;
                break;
            }
        } else {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }
    if !passed {
        panic!("Could not close the position on the exchange");
    }

    Ok(())
}
