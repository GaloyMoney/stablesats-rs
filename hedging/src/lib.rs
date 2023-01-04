#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod adjustment_action;
mod app;
mod error;
mod job;
mod okex_orders;
mod okex_transfers;
mod rebalance_action;
mod synth_usd_liability;

use galoy_client::GaloyClientConfig;
use shared::{exchanges_config::OkexConfig, health::HealthCheckTrigger, payload::*, pubsub::*};

pub use app::*;
pub use error::*;

pub async fn run(
    health_check_trigger: HealthCheckTrigger,
    config: HedgingAppConfig,
    okex_config: OkexConfig,
    galoy_config: GaloyClientConfig,
    pubsub_cfg: PubSubConfig,
    tick_receiver: memory::Subscriber<OkexBtcUsdSwapPricePayload>,
) -> Result<(), HedgingError> {
    HedgingApp::run(
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
