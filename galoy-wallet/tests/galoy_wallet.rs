use galoy_wallet::{transactions_list::TxStatus, *};
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

/// Test to retrieve the wallet IDs of the default wallets
#[tokio::test]
async fn wallets_ids() -> anyhow::Result<()> {
    let config = staging_wallet_configuration();
    let mut wallet_client = GaloyClient::new(config);
    let _ = wallet_client.login().await?;

    let wallets = wallet_client
        .wallets()
        .await?
        .expect("Expected BTC/USD wallets, found none");

    let (btc_wallet_id, usd_wallet_id) = (
        wallets.btc_wallet.expect("No BTC wallet").id,
        wallets.usd_wallet.expect("No USD wallet").id,
    );

    assert_eq!(btc_wallet_id.len(), 36);
    assert_eq!(usd_wallet_id.len(), 36);

    Ok(())
}

/// Test to get the transactions list of the default wallet
#[tokio::test]
async fn btc_transactions_list() -> anyhow::Result<()> {
    let config = staging_wallet_configuration();
    let mut wallet_client = GaloyClient::new(config);
    let _ = wallet_client.login().await?;

    let last_transaction_cursor = "5f88cbf56d87e9001ca7dab4".to_string();
    let wallet_currency = transactions_list::WalletCurrency::BTC;

    let btc_transactions = wallet_client
        .transactions_list(last_transaction_cursor, wallet_currency)
        .await?;

    let btc_transaction = &btc_transactions
        .expect("Expected transactions, found none")
        .edges
        .expect("Expected edges, found none")[0];

    assert_eq!(btc_transaction.node.status, TxStatus::SUCCESS);
    assert_eq!(btc_transaction.node.settlement_amount, 17385.0);
    assert_eq!(
        btc_transaction.node.memo,
        Some("memo Invoice GQL".to_string())
    );
    assert_eq!(btc_transaction.node.created_at, 1602800356);

    Ok(())
}

/// Test to get the USD transactions list of the  wallet
#[tokio::test]
async fn usd_transactions_list() -> anyhow::Result<()> {
    let config = staging_wallet_configuration();
    let mut wallet_client = GaloyClient::new(config);
    let _auth_token = wallet_client.login().await?;

    let last_transaction_cursor = "6213a94e13a69ff20c4941bd".to_string();
    let wallet_currency = transactions_list::WalletCurrency::USD;

    let usd_transactions = wallet_client
        .transactions_list(last_transaction_cursor, wallet_currency)
        .await?;

    let usd_transaction = &usd_transactions
        .expect("Expected transactions, found none")
        .edges
        .expect("Expected edges, found none")[0];

    assert_eq!(usd_transaction.node.status, TxStatus::SUCCESS);
    assert_eq!(usd_transaction.node.settlement_amount, 775.0);
    assert_eq!(usd_transaction.node.memo, None);
    assert_eq!(usd_transaction.node.created_at, 1645455527);

    Ok(())
}
