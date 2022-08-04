#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod app;
mod exchange_price_cache;
mod server;

use shared::pubsub::PubSubConfig;

use app::PriceApp;
pub use server::*;

pub async fn run(
    server_config: PriceServerConfig,
    pubsub_cfg: PubSubConfig,
) -> Result<(), PriceServerError> {
    let app = PriceApp::run(pubsub_cfg).await?;

    server::start(server_config, app).await?;

    Ok(())
}
