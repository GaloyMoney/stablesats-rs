use std::env;

use deribit_client::*;

use serial_test::serial;
use shared::exchanges_config::DeribitConfig;

async fn configured_client() -> anyhow::Result<DeribitClient> {
    let api_key = env::var("DERIBIT_API_KEY")?;
    let secret_key = env::var("DERIBIT_SECRET_KEY")?;

    let client = DeribitClient::new(DeribitConfig {
        api_key,
        secret_key,
        simulated: true,
    })
    .await?;

    Ok(client)
}

#[tokio::test]
#[serial]
async fn get_last_price_in_usd_cents() -> anyhow::Result<()> {
    if let Ok(client) = configured_client().await {
        let last_price = client.get_last_price_in_usd_cents().await?;

        assert!(!last_price.usd_cents.is_zero());
        assert!(last_price.usd_cents.is_sign_positive());
    } else {
        panic!("Client not configured");
    }

    Ok(())
}

#[tokio::test]
#[serial]
async fn get_btc_on_chain_deposit_address() -> anyhow::Result<()> {
    if let Ok(client) = configured_client().await {
        let address = client.get_btc_on_chain_deposit_address().await?;

        assert_eq!(address.currency, Currency::BTC.to_string());
    } else {
        panic!("Client not configured");
    }

    Ok(())
}

#[tokio::test]
#[serial]
async fn get_deposits() -> anyhow::Result<()> {
    if let Ok(client) = configured_client().await {
        let deposits = client.get_deposits().await?;

        assert_eq!(deposits.currency, Currency::BTC.to_string());
    } else {
        panic!("Client not configured");
    }

    Ok(())
}
