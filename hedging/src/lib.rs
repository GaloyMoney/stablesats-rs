#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod app;
mod config;
mod error;
pub(crate) mod hack_user_trades_lag;
mod okex;

use bria_client::BriaClientConfig;
use galoy_client::GaloyClientConfig;
use shared::{health::HealthCheckTrigger, payload::*, pubsub::*};

pub use app::*;
pub use config::*;
pub use error::*;
pub use okex::OkexConfig;

#[allow(clippy::too_many_arguments)]
pub async fn run(
    pool: sqlx::PgPool,
    health_check_trigger: HealthCheckTrigger,
    config: HedgingAppConfig,
    okex_config: OkexConfig,
    galoy_config: GaloyClientConfig,
    bria_config: BriaClientConfig,
    tick_receiver: memory::Subscriber<PriceStreamPayload>,
) -> Result<(), HedgingError> {
    HedgingApp::run(
        pool,
        health_check_trigger,
        config,
        okex_config,
        galoy_config,
        bria_config,
        tick_receiver,
    )
    .await?;
    Ok(())
}
