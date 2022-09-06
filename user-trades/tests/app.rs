use futures::StreamExt;
use rust_decimal_macros::dec;
use serial_test::serial;

use std::env;

use galoy_client::GaloyClientConfig;
use shared::{payload::*, pubsub::*};

use user_trades::*;

fn galoy_client_configuration() -> GaloyClientConfig {
    let api = env::var("GALOY_GRAPHQL_URI").expect("GALOY_GRAPHQL_URI not set");
    let phone_number = env::var("PHONE_NUMBER").expect("PHONE_NUMBER not set");
    let code = env::var("AUTH_CODE").expect("AUTH_CODE not set");

    let config = GaloyClientConfig {
        api,
        phone_number,
        code,
    };

    config
}

#[tokio::test]
#[serial]
async fn publishes_liability() -> anyhow::Result<()> {
    let redis_host = std::env::var("REDIS_HOST").unwrap_or("localhost".to_string());
    let pubsub_config = PubSubConfig {
        host: Some(redis_host),
        ..PubSubConfig::default()
    };
    let subscriber = Subscriber::new(pubsub_config.clone()).await?;
    let user_trades_pg_host =
        std::env::var("USER_TRADES_PG_HOST").unwrap_or("localhost".to_string());
    let pg_con = format!(
        "postgres://stablesats:stablesats@{}:5432/stablesats-user-trades",
        user_trades_pg_host
    );
    UserTradesApp::run(
        UserTradesAppConfig {
            migrate_on_start: true,
            pg_con,
            publish_frequency: std::time::Duration::from_millis(100),
        },
        pubsub_config,
        galoy_client_configuration(),
    )
    .await?;

    let mut stream = subscriber.subscribe::<SynthUsdLiabilityPayload>().await?;
    let received = stream.next().await.expect("expected liability message");
    assert!(received.payload.liability >= dec!(0));

    Ok(())
}
