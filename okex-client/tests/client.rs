use std::env;

use okex_client::{
    OkexClient, OkexClientConfig, OkexClientError, OKEX_MINIMUM_WITHDRAWAL_AMOUNT,
    OKEX_MINIMUM_WITHDRAWAL_FEE,
};

#[tokio::test]
async fn get_deposit_address_data() -> anyhow::Result<()> {
    let api_key = env::var("OKEX_API_KEY").expect("OKEX_API_KEY not set");
    let passphrase = env::var("OKEX_PASSPHRASE").expect("OKEX_PASS_PHRASE not set");
    let secret_key = env::var("OKEX_SECRET_KEY").expect("OKEX_SECRET_KEY not set");
    let client = OkexClient::new(OkexClientConfig {
        api_key,
        passphrase,
        secret_key,
    });
    let address = client.get_funding_deposit_address().await?;
    assert!(address.value.len() > 10);

    Ok(())
}

#[tokio::test]
async fn client_is_missing_header() -> anyhow::Result<()> {
    let client = OkexClient::new(OkexClientConfig {
        api_key: "".to_string(),
        passphrase: "".to_string(),
        secret_key: "".to_string(),
    });

    let address = client.get_funding_deposit_address().await;
    assert!(address.is_err());
    if let Err(OkexClientError::UnexpectedResponse { msg, .. }) = address {
        assert!(msg.contains("header"));
    } else {
        assert!(false)
    }

    Ok(())
}

#[tokio::test]
async fn transfer_funding_to_trading() -> anyhow::Result<()> {
    let api_key = env::var("OKEX_API_KEY").expect("OKEX_API_KEY not set");
    let passphrase = env::var("OKEX_PASSPHRASE").expect("OKEX_PASS_PHRASE not set");
    let secret_key = env::var("OKEX_SECRET_KEY").expect("OKEX_SECRET_KEY not set");
    let client = OkexClient::new(OkexClientConfig {
        api_key,
        passphrase,
        secret_key,
    });
    let amount = 0.00001;
    let transfer_id = client.transfer_funding_to_trading(amount).await?;

    assert!(transfer_id.value.len() == 9);

    Ok(())
}

#[tokio::test]
async fn transfer_trading_to_funding() -> anyhow::Result<()> {
    let api_key = env::var("OKEX_API_KEY").expect("OKEX_API_KEY not set");
    let passphrase = env::var("OKEX_PASSPHRASE").expect("OKEX_PASS_PHRASE not set");
    let secret_key = env::var("OKEX_SECRET_KEY").expect("OKEX_SECRET_KEY not set");
    let client = OkexClient::new(OkexClientConfig {
        api_key,
        passphrase,
        secret_key,
    });
    let amount = 0.00001;
    let transfer_id = client.transfer_trading_to_funding(amount).await?;

    assert!(transfer_id.value.len() == 9);

    Ok(())
}

#[tokio::test]
async fn funding_account_balance() -> anyhow::Result<()> {
    let api_key = env::var("OKEX_API_KEY").expect("OKEX_API_KEY not set");
    let passphrase = env::var("OKEX_PASSPHRASE").expect("OKEX_PASS_PHRASE not set");
    let secret_key = env::var("OKEX_SECRET_KEY").expect("OKEX_SECRET_KEY not set");
    let client = OkexClient::new(OkexClientConfig {
        api_key,
        passphrase,
        secret_key,
    });

    let avail_balance = client.funding_account_balance().await?;
    let balance = avail_balance.value.parse::<f64>()?;

    assert!(balance >= 0.00);

    Ok(())
}

#[tokio::test]
async fn trading_account_balance() -> anyhow::Result<()> {
    let api_key = env::var("OKEX_API_KEY").expect("OKEX_API_KEY not set");
    let passphrase = env::var("OKEX_PASSPHRASE").expect("OKEX_PASS_PHRASE not set");
    let secret_key = env::var("OKEX_SECRET_KEY").expect("OKEX_SECRET_KEY not set");
    let client = OkexClient::new(OkexClientConfig {
        api_key,
        passphrase,
        secret_key,
    });

    let avail_balance = client.trading_account_balance().await?;
    let balance = avail_balance.value.parse::<f64>()?;

    assert!(balance >= 0.00);

    Ok(())
}

#[tokio::test]
async fn transfer_state() -> anyhow::Result<()> {
    let api_key = env::var("OKEX_API_KEY").expect("OKEX_API_KEY not set");
    let passphrase = env::var("OKEX_PASSPHRASE").expect("OKEX_PASS_PHRASE not set");
    let secret_key = env::var("OKEX_SECRET_KEY").expect("OKEX_SECRET_KEY not set");
    let client = OkexClient::new(OkexClientConfig {
        api_key,
        passphrase,
        secret_key,
    });
    let amount = 0.00001;
    let transfer_id = client.transfer_funding_to_trading(amount).await?;

    let transfer_state = client.transfer_state(transfer_id).await?;

    assert_eq!(transfer_state.value, "success".to_string());

    Ok(())
}

#[tokio::test]
async fn withdraw_to_onchain_address() -> anyhow::Result<()> {
    let api_key = env::var("OKEX_API_KEY").expect("OKEX_API_KEY not set");
    let passphrase = env::var("OKEX_PASSPHRASE").expect("OKEX_PASS_PHRASE not set");
    let secret_key = env::var("OKEX_SECRET_KEY").expect("OKEX_SECRET_KEY not set");
    let client = OkexClient::new(OkexClientConfig {
        api_key,
        passphrase,
        secret_key,
    });

    let amount = OKEX_MINIMUM_WITHDRAWAL_AMOUNT;
    let fee = OKEX_MINIMUM_WITHDRAWAL_FEE;
    let onchain_address = "bc1qafuzw5ga4perwsugcmaecjc5epydsqaj7cwk7j".to_string();
    let withdraw_id = client
        .withdraw_btc_onchain(amount, fee, onchain_address)
        .await?;

    assert!(withdraw_id.value.len() == 8);

    Ok(())
}

#[tokio::test]
async fn place_order() -> anyhow::Result<()> {
    let api_key = env::var("OKEX_DEMO_API_KEY").expect("OKEX_API_KEY not set");
    let passphrase = env::var("OKEX_DEMO_PASSPHRASE").expect("OKEX_PASS_PHRASE not set");
    let secret_key = env::var("OKEX_DEMO_SECRET_KEY").expect("OKEX_SECRET_KEY not set");
    let client = OkexClient::new(OkexClientConfig {
        api_key,
        passphrase,
        secret_key,
    });

    let order_id = client
        .place_order(
            "BTC-USD-SWAP".to_string(),
            "cross".to_string(),
            "buy".to_string(),
            "long".to_string(),
            "market".to_string(),
            1,
        )
        .await?;

    assert!(order_id.value.len() == 8);

    Ok(())
}

#[tokio::test]
async fn get_positions() -> anyhow::Result<()> {
    let api_key = env::var("OKEX_DEMO_API_KEY").expect("OKEX_API_KEY not set");
    let passphrase = env::var("OKEX_DEMO_PASSPHRASE").expect("OKEX_PASS_PHRASE not set");
    let secret_key = env::var("OKEX_DEMO_SECRET_KEY").expect("OKEX_SECRET_KEY not set");
    let client = OkexClient::new(OkexClientConfig {
        api_key,
        passphrase,
        secret_key,
    });

    let position = client.position().await?;

    assert_eq!(position, "position");

    Ok(())
}
