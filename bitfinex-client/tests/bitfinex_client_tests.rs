use std::env;

use bitfinex_client::*;

use serial_test::serial;

async fn configured_client() -> anyhow::Result<BitfinexClient> {
    let api_key = env::var("BITFINEX_API_KEY")?;
    let secret_key = env::var("BITFINEX_SECRET_KEY")?;

    let client = BitfinexClient::new(BitfinexConfig {
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
    }

    Ok(())
}

#[tokio::test]
#[serial]
async fn funding_info() -> anyhow::Result<()> {
    if let Ok(client) = configured_client().await {
        let info = client.funding_info().await?;

        assert!(info.yield_lend.is_zero());
    }

    Ok(())
}

#[tokio::test]
#[serial]
async fn get_orders() -> anyhow::Result<()> {
    if let Ok(client) = configured_client().await {
        let _orders = client.get_orders().await?;
    }

    Ok(())
}

#[tokio::test]
#[serial]
async fn positions() -> anyhow::Result<()> {
    if let Ok(client) = configured_client().await {
        let _positions = client.get_positions().await?;
    }

    Ok(())
}

#[tokio::test]
#[serial]
async fn close_position() -> anyhow::Result<()> {
    if let Ok(client) = configured_client().await {
        let _close_position = client.close_position().await?;
    }

    Ok(())
}

#[tokio::test]
#[serial]
async fn funding_account_balance() -> anyhow::Result<()> {
    if let Ok(client) = configured_client().await {
        let _funding_account_balance = client.funding_account_balance().await?;
    }

    Ok(())
}

#[tokio::test]
#[serial]
async fn trading_account_balance() -> anyhow::Result<()> {
    if let Ok(client) = configured_client().await {
        let _trading_account_balance = client.trading_account_balance().await?;
    }

    Ok(())
}

#[tokio::test]
#[serial]
async fn get_funding_deposit_address() -> anyhow::Result<()> {
    if let Ok(client) = configured_client().await {
        let _address = client.get_funding_deposit_address().await?;
    }

    Ok(())
}

#[tokio::test]
#[serial]
async fn get_btc_on_chain_transactions() -> anyhow::Result<()> {
    if let Ok(client) = configured_client().await {
        let _transactions = client.get_btc_on_chain_transactions().await?;
    }

    Ok(())
}

#[tokio::test]
#[serial]
async fn get_api_key_permissions() -> anyhow::Result<()> {
    if let Ok(client) = configured_client().await {
        let _keys = client.get_api_key_permissions().await?;
    }

    Ok(())
}
