use rust_decimal_macros::dec;
use serial_test::serial;

use std::env;

use galoy_client::GaloyClientConfig;

use ::user_trades::*;

fn galoy_client_configuration() -> GaloyClientConfig {
    let api = env::var("GALOY_GRAPHQL_URI").expect("GALOY_GRAPHQL_URI not set");
    let phone_number = env::var("PHONE_NUMBER").expect("PHONE_NUMBER not set");
    let code = env::var("AUTH_CODE").expect("AUTH_CODE not set");

    let config = GaloyClientConfig {
        api,
        phone_number,
        auth_code: code,
    };

    config
}

#[tokio::test]
#[serial]
async fn publishes_liability() -> anyhow::Result<()> {
    let pg_host = std::env::var("PG_HOST").unwrap_or("localhost".to_string());
    let pg_con = format!("postgres://user:password@{pg_host}:5432/pg",);
    let pool = sqlx::PgPool::connect(&pg_con).await?;
    let mut events = ledger::Ledger::init(&pool)
        .await?
        .usd_liability_balance_events()
        .await;
    let _ = tokio::spawn(UserTradesApp::run(
        pool,
        UserTradesConfig {
            galoy_poll_frequency: std::time::Duration::from_secs(1),
        },
        galoy_client_configuration(),
    ));

    let received = events.recv().await.unwrap();
    if let ledger::LedgerEventData::BalanceUpdated(data) = received.data {
        assert!(data.settled_dr_balance >= dec!(0));
    } else {
        assert!(false)
    }

    Ok(())
}
