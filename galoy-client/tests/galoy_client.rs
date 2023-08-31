use rust_decimal_macros::dec;
use std::env;

use galoy_client::*;

fn client_configuration() -> GaloyClientConfig {
    let api = env::var("GALOY_GRAPHQL_URI").expect("GALOY_GRAPHQL_URI not set");
    let phone_number = env::var("PHONE_NUMBER").expect("PHONE_NUMBER not set");
    let code = env::var("AUTH_CODE").expect("AUTH_CODE not set");

    let config = GaloyClientConfig {
        api,
        phone_number,
        auth_code: code,
    };

    config
}

/// Test to get transactions list of the default wallet
#[tokio::test]
async fn transactions_list() -> anyhow::Result<()> {
    let config = client_configuration();
    let client = GaloyClient::connect(config).await?;

    let transactions = client.transactions_list(None).await?;
    assert!(transactions.list.len() > 0);

    Ok(())
}

/// Test to get wallet balances
#[tokio::test]
async fn wallet_balance() -> anyhow::Result<()> {
    let config = client_configuration();
    let wallet_client = GaloyClient::connect(config).await?;

    let balances = wallet_client.wallet_balances().await?;

    assert!(balances.btc >= dec!(0));
    assert!(balances.usd <= dec!(0));

    Ok(())
}
