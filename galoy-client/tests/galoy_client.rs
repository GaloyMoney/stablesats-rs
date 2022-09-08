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
        code,
    };

    config
}

/// Test to get transactions list of the default wallet
#[tokio::test]
async fn transactions_list() -> anyhow::Result<()> {
    let config = client_configuration();
    let client = GaloyClient::connect(config).await?;

    let transactions = client.transactions_list(None).await?;
    println!("{:#?}", transactions);
    assert!(transactions.list.len() == 0);

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

/// Test to generate onchain deposit address
#[tokio::test]
async fn onchain_deposit_address() -> anyhow::Result<()> {
    let config = client_configuration();
    let wallet_client = GaloyClient::connect(config).await?;

    let onchain_address = wallet_client.onchain_address().await?;

    assert!(onchain_address.address.len() == 42);
    Ok(())
}

/// Test making an onchain payment
#[tokio::test]
#[ignore]
async fn onchain_payment() -> anyhow::Result<()> {
    let config = client_configuration();
    let wallet_client = GaloyClient::connect(config).await?;

    let onchain_address =
        env::var("ONCHAIN_DEPOSIT_ADDRESS").expect("ONCHAIN_DEPOSIT_ADDRESS not set");
    let memo = "Test onchain payment".to_string();
    let amount = dec!(1001);
    let target_conf = 2;

    let payment_result = wallet_client
        .send_onchain_payment(onchain_address, amount, Some(memo), target_conf)
        .await?;

    assert_eq!(payment_result, PaymentSendResult::SUCCESS);

    Ok(())
}

/// Test to get onchain transaction fee
#[tokio::test]
#[ignore]
async fn onchain_tx_fee() -> anyhow::Result<()> {
    let config = client_configuration();
    let wallet_client = GaloyClient::connect(config).await?;
    let testnet_onchain_tx_fee = dec!(2142);

    let onchain_address =
        env::var("ONCHAIN_DEPOSIT_ADDRESS").expect("ONCHAIN_DEPOSIT_ADDRESS not set");
    let amount = dec!(50000);
    let target_conf = 1;

    let onchain_tx_fee = wallet_client
        .onchain_tx_fee(onchain_address, amount, target_conf)
        .await?;

    assert_eq!(onchain_tx_fee.amount, testnet_onchain_tx_fee);
    Ok(())
}
