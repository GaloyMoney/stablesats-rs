#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod app;
pub mod cache;
pub mod currency;
pub mod entity;
pub mod error;
pub mod price;
pub mod quote;
pub mod server;

use shared::{health::HealthCheckTrigger, payload::*, pubsub::memory};

pub use app::*;
pub use cache::QuotesExchangePriceCacheConfig;
pub use entity::*;
pub use price::*;
pub use server::*;

pub async fn run(
    health_check_trigger: HealthCheckTrigger,
    health_check_cfg: QuotesServerHealthCheckConfig,
    fee_calc_cfg: QuotesFeeCalculatorConfig,
    subscriber: memory::Subscriber<PriceStreamPayload>,
    price_cache_config: QuotesExchangePriceCacheConfig,
    exchange_weights: ExchangeWeights,
    pool: sqlx::PgPool,
) -> Result<(), QuotesServerError> {
    let app = QuotesApp::run(
        health_check_trigger,
        health_check_cfg,
        fee_calc_cfg,
        subscriber,
        price_cache_config,
        exchange_weights,
        pool,
    )
    .await?;

    // server::start(server_config, app).await?;

    Ok(())
}
