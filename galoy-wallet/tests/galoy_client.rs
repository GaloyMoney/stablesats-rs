use futures::StreamExt;
use galoy_wallet::{stablesats_transactions_list::TxStatus, *};
use std::env;

fn staging_wallet_configuration() -> GaloyClientConfig {
    let api = env::var("GALOY_STAGING_GRAPHQL_URI").expect("GALOY_STAGING_GRAPHQL_URI not set");
    let phone_number = env::var("STAGING_PHONE_NUMBER").expect("STAGING_PHONE_NUMBER not set");
    let code = env::var("GALOY_STAGING_AUTH_CODE").expect("GALOY_STAGING_AUTH_CODE not set");

    let config = GaloyClientConfig {
        api,
        phone_number,
        code,
    };

    config
}

/// Test to get btc transactions list of the default wallet
#[tokio::test]
async fn btc_transactions_list() -> anyhow::Result<()> {
    let config = staging_wallet_configuration();
    let mut wallet_client = GaloyClient::connect(config).await?;

    let last_transaction_cursor = "5f88cbf56d87e9001ca7dab4".to_string();
    let wallet_currency = stablesats_transactions_list::WalletCurrency::BTC;

    let mut btc_transactions = wallet_client
        .transactions_list(last_transaction_cursor, wallet_currency)
        .await?;

    let tx_1 = btc_transactions
        .next()
        .await
        .expect("Expected transaction, found None");

    assert_eq!(tx_1.node.status, TxStatus::SUCCESS);
    assert_eq!(tx_1.node.settlement_amount, 17385.0);
    assert_eq!(tx_1.node.memo, Some("memo Invoice GQL".to_string()));
    assert_eq!(tx_1.node.created_at, 1602800356);
    assert_eq!(tx_1.cursor, "5f88cae4633400001c0ea321");

    Ok(())
}

/// Test to get the USD transactions list of the default wallet
#[tokio::test]
async fn usd_transactions_list() -> anyhow::Result<()> {
    let config = staging_wallet_configuration();
    let mut wallet_client = GaloyClient::connect(config).await?;

    let last_transaction_cursor = "6213a94e13a69ff20c4941bd".to_string();
    let wallet_currency = stablesats_transactions_list::WalletCurrency::USD;

    let mut usd_transactions = wallet_client
        .transactions_list(last_transaction_cursor, wallet_currency)
        .await?;

    let tx_1 = usd_transactions
        .next()
        .await
        .expect("Expected transaction, found None");

    assert_eq!(tx_1.cursor, "6213a8a76208f84146869625");
    assert_eq!(tx_1.node.status, TxStatus::SUCCESS);
    assert_eq!(tx_1.node.settlement_amount, 775.0);
    assert_eq!(tx_1.node.memo, None);
    assert_eq!(tx_1.node.created_at, 1645455527);

    Ok(())
}

/// Test to get wallet balances
#[tokio::test]
async fn wallet_balance() -> anyhow::Result<()> {
    let config = staging_wallet_configuration();
    let wallet_client = GaloyClient::connect(config).await?;

    let balances = wallet_client.wallets_balances().await?;

    assert!(balances.btc_wallet_balance.is_some());
    assert!(balances.usd_wallet_balance.is_some());

    Ok(())
}
