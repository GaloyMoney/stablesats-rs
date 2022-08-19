#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod app;
pub mod user_trade;
pub mod user_trade_balances;

use shared::pubsub::*;

pub use app::*;

pub async fn run(
    config: UserTradesAppConfig,
    pubsub_cfg: PubSubConfig,
) -> Result<(), UserTradesAppError> {
    UserTradesApp::run(config, pubsub_cfg).await?;
    Ok(())
}

pub async fn migrate(pg_con: &str) -> anyhow::Result<()> {
    use sqlx::Connection;
    let mut con = sqlx::PgConnection::connect(pg_con).await?;
    sqlx::migrate!().run(&mut con).await?;
    Ok(())
}
