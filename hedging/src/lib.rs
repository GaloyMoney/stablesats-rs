#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod adjustment_action;
mod app;
mod config;
mod error;
pub(crate) mod hack_user_trades_lag;
mod okex_orders;
mod okex_transfers;
mod rebalance_action;

pub mod job;

use galoy_client::GaloyClientConfig;
use shared::{exchanges_config::OkexConfig, health::HealthCheckTrigger, payload::*, pubsub::*};

pub use app::*;
pub use config::*;
pub use error::*;

#[allow(clippy::too_many_arguments)]
pub async fn run(
    pool: sqlx::PgPool,
    health_check_trigger: HealthCheckTrigger,
    config: HedgingAppConfig,
    okex_config: OkexConfig,
    galoy_config: GaloyClientConfig,
    pubsub_cfg: PubSubConfig,
    tick_receiver: memory::Subscriber<PriceStreamPayload>,
) -> Result<(), HedgingError> {
    HedgingApp::run(
        pool,
        health_check_trigger,
        config,
        okex_config,
        galoy_config,
        pubsub_cfg,
        tick_receiver,
    )
    .await?;
    Ok(())
}
