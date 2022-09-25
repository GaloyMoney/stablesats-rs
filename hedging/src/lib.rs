#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod adjustment_action;
mod app;
mod error;
mod hedging_adjustments;
mod job;
mod synth_usd_liability;

use okex_client::OkexClientConfig;
use shared::{health::HealthCheckTrigger, pubsub::*};

pub use app::*;
pub use error::*;

pub async fn run(
    health_check_trigger: HealthCheckTrigger,
    config: HedgingAppConfig,
    okex_config: OkexClientConfig,
    pubsub_cfg: PubSubConfig,
) -> Result<(), HedgingError> {
    HedgingApp::run(health_check_trigger, config, okex_config, pubsub_cfg).await?;
    Ok(())
}
