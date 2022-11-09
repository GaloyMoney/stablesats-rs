#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod app;
pub mod currency;
mod exchange_tick_cache;
mod fee_calculator;
mod order_book_cache;
mod price_converter;
mod price_mixer;
mod server;

use shared::{
    exchanges_config::ExchangeConfigAll, health::HealthCheckTrigger, pubsub::PubSubConfig,
};

use app::PriceApp;
pub use fee_calculator::FeeCalculatorConfig;
pub use order_book_cache::*;
pub use price_converter::*;
pub use server::*;

pub use price_mixer::ExchangePriceCacheError;

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
