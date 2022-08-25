use galoy_wallet::*;
use std::env;

fn wallet_configuration() -> GaloyClientConfig {
    let api = env::var("GALOY_GRAPHQL_URI").expect("GALOY_GRAPHQL_URI not set");
    let phone_number = env::var("WALLET_PHONE_NUMBER").expect("WALLET_PHONE_NUMBER not set");
    let config = GaloyClientConfig { api, phone_number };

    config
}

/// Test send an non-authenticated query to the Galoy Graphql API
#[tokio::test]
async fn get_btc_price() -> anyhow::Result<()> {
    let config = wallet_configuration();

    let wallet_client = GaloyClient::new(config);
    let price = wallet_client.btc_price().await?;

    println!("{:#?}", price);

    assert_eq!(price.offset, 12);
    assert_eq!(
        price.currency_unit,
        btc_price::ExchangeCurrencyUnit::USDCENT
    );

    Ok(())
}

/// Test getting an authentication code from the Galoy Graphql API
#[tokio::test]
async fn authentication_code() -> anyhow::Result<()> {
    let config = wallet_configuration();

    let wallet_client = GaloyClient::new(config);
    let auth_code_response = wallet_client.authentication_code().await?;

    assert_eq!(
        auth_code_response,
        auth_code::AuthCodeUserRequestAuthCode {
            success: Some(true),
            errors: vec![auth_code::AuthCodeUserRequestAuthCodeErrors {
                message: "".to_string()
            }]
        }
    );

    Ok(())
}

/// Test login to account
#[tokio::test]
async fn login() -> anyhow::Result<()> {
    let config = wallet_configuration();

    let wallet_client = GaloyClient::new(config);

    let auth_code = "123456".to_string();
    let login_response = wallet_client.login(auth_code).await?;

    let auth_token = login_response.auth_token.expect("Empty auth token");

    assert_eq!(auth_token.len(), 50);

    Ok(())
}

/// Test getting the public wallet account
#[tokio::test]
async fn public_wallet() -> anyhow::Result<()> {
    let config = wallet_configuration();

    let wallet_client = GaloyClient::new(config);
    let test_username = "test".to_string();
    let _public_wallet = wallet_client.public_wallet(test_username).await?;

    Ok(())
}
