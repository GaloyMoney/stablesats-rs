use galoy_wallet::*;
use std::env;

fn staging_wallet_configuration() -> GaloyClientConfig {
    let api = env::var("GALOY_STAGING_GRAPHQL_URI").expect("GALOY_STAGING_GRAPHQL_URI not set");
    let phone_number = env::var("STAGING_PHONE_NUMBER").expect("STAGING_PHONE_NUMBER not set");
    let code = env::var("GALOY_STAGING_AUTH_CODE").expect("GALOY_STAGING_AUTH_CODE not set");
    let jwt = "".to_string();

    let config = GaloyClientConfig {
        api,
        phone_number,
        code,
        jwt,
    };

    config
}

fn dev_wallet_configuration() -> GaloyClientConfig {
    let api = env::var("GALOY_DEV_GRAPHQL_URI").expect("GALOY_DEV_GRAPHQL_URI not set");
    let phone_number = env::var("DEV_PHONE_NUMBER").expect("DEV_PHONE_NUMBER not set");
    let code = env::var("GALOY_DEV_AUTH_CODE").expect("GALOY_DEV_AUTH_CODE not set");
    let jwt = "".to_string();

    let config = GaloyClientConfig {
        api,
        phone_number,
        code,
        jwt,
    };

    config
}

/// Test send an non-authenticated query to the Galoy Graphql API
#[tokio::test]
async fn get_btc_price() -> anyhow::Result<()> {
    let config = dev_wallet_configuration();

    let wallet_client = GaloyClient::new(config);
    let price = wallet_client.btc_price().await?;

    assert_eq!(price.offset, 12);
    assert_eq!(
        price.currency_unit,
        btc_price::ExchangeCurrencyUnit::USDCENT
    );

    Ok(())
}

/// Test getting an authentication code from the Galoy Graphql API
#[tokio::test]
#[ignore = "Need region-enabled/known phone service provider"]
async fn authentication_code() -> anyhow::Result<()> {
    let config = dev_wallet_configuration();

    let wallet_client = GaloyClient::new(config);
    let auth_code_response = wallet_client.authentication_code().await?;

    assert_eq!(
        auth_code_response,
        auth_code::AuthCodeUserRequestAuthCode {
            success: Some(true),
            errors: vec![]
        }
    );

    Ok(())
}

/// Test login to account
#[tokio::test]
async fn login() -> anyhow::Result<()> {
    let config = dev_wallet_configuration();
    let mut wallet_client = GaloyClient::new(config);

    let login_response = wallet_client.login().await?;
    let auth_token = login_response.auth_token.expect("Empty auth token");

    assert_eq!(auth_token.len(), 176);

    Ok(())
}

/// Test to get the transactions list of the default wallet
#[tokio::test]
async fn transactions_list() -> anyhow::Result<()> {
    let config = staging_wallet_configuration();
    let mut wallet_client = GaloyClient::new(config);
    let _auth_token = wallet_client.login().await?;

    let last_transaction_cursor = Some("YXJyYXljb25uZWN0aW9uOjQwNg==".to_string());
    let transactions = wallet_client
        .transactions_list(last_transaction_cursor)
        .await?;

    println!("{:#?}", transactions);
    // assert_eq!(edges.len(), 10);
    Ok(())
}

/// Test to get the btc transactions list of the  wallet
#[tokio::test]
async fn btc_transactions_list() -> anyhow::Result<()> {
    let config = staging_wallet_configuration();
    let mut wallet_client = GaloyClient::new(config);
    let _auth_token = wallet_client.login().await?;

    let btc_transactions = wallet_client.btc_transactions_list(None).await?;

    println!("{:#?}", btc_transactions);
    // assert_eq!(edges.len(), 10);
    Ok(())
}
