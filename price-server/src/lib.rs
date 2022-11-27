#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod app;
pub mod currency;
mod exchange_price_cache;
mod exchange_tick_cache;
mod fee_calculator;
mod price_mixer;
mod server;

use shared::{
    exchanges_config::ExchangeConfigAll, health::HealthCheckTrigger, pubsub::PubSubConfig,
};

use app::PriceApp;
pub use exchange_price_cache::ExchangePriceCacheError;
pub use fee_calculator::FeeCalculatorConfig;
pub use server::*;

pub async fn run(
    health_check_trigger: HealthCheckTrigger,
    server_config: PriceServerConfig,
    fee_calc_cfg: FeeCalculatorConfig,
    pubsub_cfg: PubSubConfig,
    exchanges_cfg: ExchangeConfigAll,
) -> Result<(), PriceServerError> {
    let app = PriceApp::run(
        health_check_trigger,
        fee_calc_cfg,
        pubsub_cfg,
        exchanges_cfg,
    )
    .await?;

    server::start(server_config, app).await?;

    Ok(())
}
