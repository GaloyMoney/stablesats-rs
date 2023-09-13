#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod app;
pub mod error;
pub mod server;

pub use app::*;
pub use server::*;

pub async fn run(// health_check_trigger: HealthCheckTrigger,
) -> Result<(), QuotesServerError> {
    let app = QuotesApp::run(
        // health_check_trigger,
        // health_check_cfg,
        // fee_calc_cfg,
        // subscriber,
        // price_cache_config,
        // exchange_weights,
    )
    .await?;

    // server::start(server_config, app).await?;

    Ok(())
}
