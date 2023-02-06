#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod app;
mod error;
mod galoy_transactions;
pub mod job;
pub mod user_trade_unit;
pub mod user_trades;

use galoy_client::GaloyClientConfig;

pub use app::*;
pub use error::*;

pub async fn run(
    pool: sqlx::PgPool,
    config: UserTradesConfig,
    galoy_client_cfg: GaloyClientConfig,
) -> Result<(), UserTradesError> {
    UserTradesApp::run(pool, config, galoy_client_cfg).await?;
    Ok(())
}
