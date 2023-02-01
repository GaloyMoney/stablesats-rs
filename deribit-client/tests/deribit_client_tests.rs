use std::env;

use deribit_client::*;

use rust_decimal_macros::dec;
use serial_test::serial;
use shared::exchanges_config::DeribitConfig;

async fn configured_client() -> anyhow::Result<DeribitClient> {
    let funding_api_key = env::var("FUNDING_DERIBIT_API_KEY")?;
    let funding_secret_key = env::var("FUNDING_DERIBIT_SECRET_KEY")?;
    let trading_api_key = env::var("TRADING_DERIBIT_API_KEY")?;
    let trading_secret_key = env::var("TRADING_DERIBIT_SECRET_KEY")?;

    let client = DeribitClient::new(DeribitConfig {
        funding_api_key,
        funding_secret_key,
        trading_api_key,
        trading_secret_key,
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

        assert_eq!(deposits[0].currency, Currency::BTC.to_string());
    } else {
        panic!("Client not configured");
    }

    Ok(())
}

#[tokio::test]
#[serial]
async fn get_transfers() -> anyhow::Result<()> {
    if let Ok(client) = configured_client().await {
        let transfers = client.get_transfers().await?;

        assert_eq!(transfers[0].currency, Currency::BTC.to_string());
    } else {
        panic!("Client not configured");
    }

    Ok(())
}

#[tokio::test]
#[serial]
async fn get_withdrawals() -> anyhow::Result<()> {
    if let Ok(client) = configured_client().await {
        let withdrawals = client.get_withdrawals().await?;

        if !withdrawals.is_empty() {
            assert_eq!(withdrawals[0].currency, Currency::BTC.to_string());
        }
    } else {
        panic!("Client not configured");
    }

    Ok(())
}

#[tokio::test]
#[serial]
async fn buy() -> anyhow::Result<()> {
    if let Ok(client) = configured_client().await {
        let client_id = ClientId::new();
        let amount = dec!(10);
        let buy = client.buy(client_id, amount).await?;

        assert_eq!(buy.amount, amount);

        let client_id = ClientId::new();
        client.close_position(client_id).await?;
    } else {
        panic!("Client not configured");
    }

    Ok(())
}

#[tokio::test]
#[serial]
async fn sell() -> anyhow::Result<()> {
    if let Ok(client) = configured_client().await {
        let client_id = ClientId::new();
        let amount = dec!(10);
        let sell = client.sell(client_id, amount).await?;

        assert_eq!(sell.amount, amount);

        let client_id = ClientId::new();
        client.close_position(client_id).await?;
    } else {
        panic!("Client not configured");
    }

    Ok(())
}

#[tokio::test]
#[serial]
async fn get_order_state() -> anyhow::Result<()> {
    if let Ok(client) = configured_client().await {
        let client_id = ClientId::new();
        let amount = dec!(10);
        let sell = client.sell(client_id, amount).await?;

        assert_eq!(sell.amount, amount);

        let order_state = client.get_order_state(sell.order_id).await?;
        assert_eq!(order_state.amount, amount);

        let client_id = ClientId::new();
        client.close_position(client_id).await?;
    } else {
        panic!("Client not configured");
    }

    Ok(())
}

#[tokio::test]
#[serial]
async fn get_position() -> anyhow::Result<()> {
    if let Ok(client) = configured_client().await {
        let client_id = ClientId::new();
        let _ = client.close_position(client_id).await;

        let client_id = ClientId::new();
        let amount = dec!(10);
        let sell = client.sell(client_id.clone(), amount).await?;

        assert_eq!(sell.amount, amount);

        let position = client.get_position().await?;
        assert_eq!(position.size.abs(), amount);

        let client_id = ClientId::new();
        client.close_position(client_id).await?;
    } else {
        panic!("Client not configured");
    }

    Ok(())
}

#[tokio::test]
#[serial]
async fn get_funding_account_summary() -> anyhow::Result<()> {
    if let Ok(client) = configured_client().await {
        let account_summary = client.get_funding_account_summary().await?;
        assert_eq!(account_summary.currency, Currency::BTC.to_string());
    } else {
        panic!("Client not configured");
    }

    Ok(())
}

#[tokio::test]
#[serial]
async fn get_trading_account_summary() -> anyhow::Result<()> {
    if let Ok(client) = configured_client().await {
        let account_summary = client.get_trading_account_summary().await?;
        assert_eq!(account_summary.currency, Currency::BTC.to_string());
    } else {
        panic!("Client not configured");
    }

    Ok(())
}
