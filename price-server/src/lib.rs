#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod app;
pub mod currency;
mod exchange_price_cache;
mod fee_calculator;
mod order_book_cache;
mod price_converter;
mod server;

use shared::{health::HealthCheckTrigger, pubsub::PubSubConfig};

use app::PriceApp;
pub use exchange_price_cache::ExchangePriceCacheError;
pub use fee_calculator::FeeCalculatorConfig;
pub use order_book_cache::*;
pub use price_converter::*;
pub use server::*;

pub async fn run(
    health_check_trigger: HealthCheckTrigger,
    server_config: PriceServerConfig,
    fee_calc_cfg: FeeCalculatorConfig,
    pubsub_cfg: PubSubConfig,
) -> Result<(), PriceServerError> {
    let app = PriceApp::run(health_check_trigger, fee_calc_cfg, pubsub_cfg).await?;

    server::start(server_config, app).await?;

    Ok(())
}
