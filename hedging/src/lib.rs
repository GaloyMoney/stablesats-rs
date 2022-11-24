#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod adjustment_action;
mod app;
mod error;
mod job;
mod okex_orders;
mod synth_usd_liability;

use shared::{exchanges_config::OkexConfig, health::HealthCheckTrigger, pubsub::*};

pub use app::*;
pub use error::*;

pub async fn run(
    health_check_trigger: HealthCheckTrigger,
    config: HedgingAppConfig,
    okex_config: OkexConfig,
    pubsub_cfg: PubSubConfig,
) -> Result<(), HedgingError> {
    HedgingApp::run(health_check_trigger, config, okex_config, pubsub_cfg).await?;
    Ok(())
}
