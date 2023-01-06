#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod app;
mod cache_config;
pub mod currency;
mod error;
mod exchange_tick_cache;
mod fee_calculator;
mod price_mixer;
mod server;

use app::PriceApp;
use shared::{
    exchanges_config::ExchangeConfigs, health::HealthCheckTrigger, payload::*, pubsub::memory,
};

pub use app::PriceServerHealthCheckConfig;
pub use cache_config::ExchangePriceCacheConfig;
pub use fee_calculator::FeeCalculatorConfig;
pub use server::*;

pub async fn run(
    health_check_trigger: HealthCheckTrigger,
    health_check_cfg: PriceServerHealthCheckConfig,
    server_config: PriceServerConfig,
    fee_calc_cfg: FeeCalculatorConfig,
    subscriber: memory::Subscriber<PriceStreamPayload>,
    price_cache_config: ExchangePriceCacheConfig,
    exchanges_cfg: ExchangeConfigs,
) -> Result<(), PriceServerError> {
    let app = PriceApp::run(
        health_check_trigger,
        health_check_cfg,
        fee_calc_cfg,
        subscriber,
        price_cache_config,
        exchanges_cfg,
    )
    .await?;

    server::start(server_config, app).await?;

    Ok(())
}
