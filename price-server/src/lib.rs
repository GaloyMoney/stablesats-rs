#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod app;
mod exchange_price_cache;
mod fee_calculator;
mod server;
mod sat_cent_converter;
mod cent_usd_converter;

use shared::pubsub::PubSubConfig;

use app::PriceApp;
pub use fee_calculator::FeeCalculatorConfig;
pub use server::*;

pub async fn run(
    server_config: PriceServerConfig,
    fee_calc_cfg: FeeCalculatorConfig,
    pubsub_cfg: PubSubConfig,
) -> Result<(), PriceServerError> {
    let app = PriceApp::run(fee_calc_cfg, pubsub_cfg).await?;

    server::start(server_config, app).await?;

    Ok(())
}
