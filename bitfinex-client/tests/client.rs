use std::env;

use bitfinex_client::*;

use serial_test::serial;
use tracing::debug;

async fn configured_client() -> anyhow::Result<BitfinexClient> {
    let api_key = env::var("BITFINEX_API_KEY").expect("BITFINEX_API_KEY not set");
    let secret_key = env::var("BITFINEX_SECRET_KEY").expect("BITFINEX_SECRET_KEY not set");

    let client = BitfinexClient::new(BitfinexClientConfig {
        api_key,
        secret_key,
        simulated: true,
    })
    .await?;

    Ok(client)
}

#[tokio::test]
#[serial]
async fn last_price() -> anyhow::Result<()> {
    let client = configured_client().await?;

    let last_price = client.get_last_price_in_usd_cents().await?;

    debug!("bitfinex last_price is: {}", last_price.usd_cents);

    assert!(!last_price.usd_cents.is_zero());
    assert!(last_price.usd_cents.is_sign_positive());

    Ok(())
}

#[tokio::test]
#[serial]
async fn funding_info() -> anyhow::Result<()> {
    let client = configured_client().await?;

    let info = client.funding_info().await?;

    debug!(
        "bitfinex funding info on {}, {} is: {}",
        info.key, info.symbol, info.yield_lend
    );

    assert!(!info.yield_lend.is_zero());
    assert!(info.yield_lend.is_sign_positive());

    Ok(())
}
