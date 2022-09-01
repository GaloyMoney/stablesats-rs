use futures::StreamExt;
use galoy_client::{
    stablesats_transactions_list::{TxStatus, WalletCurrency},
    *,
};
use std::env;

fn staging_wallet_configuration() -> GaloyClientConfig {
    let api = env::var("STAGING_GRAPHQL_URI").expect("STAGING_GRAPHQL_URI not set");
    let phone_number = env::var("STAGING_PHONE_NUMBER").expect("PHONE_NUMBER not set");
    let code = env::var("STAGING_AUTH_CODE").expect("STAGING_AUTH_CODE not set");

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

    let last_transaction_cursor = LastTransactionCursor("YXJyYXljb25uZWN0aW9uOjQxMQ==".to_string());
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
    assert_eq!(tx_1.cursor, "YXJyYXljb25uZWN0aW9uOjQxMg==");

    Ok(())
}

/// Test to get the USD transactions list of the default wallet
#[tokio::test]
async fn usd_transactions_list() -> anyhow::Result<()> {
    let config = staging_wallet_configuration();
    let mut wallet_client = GaloyClient::connect(config).await?;

    let last_transaction_cursor = LastTransactionCursor("YXJyYXljb25uZWN0aW9uOjQwOA==".to_string());
    let wallet_currency = stablesats_transactions_list::WalletCurrency::USD;

    let mut usd_transactions = wallet_client
        .transactions_list(last_transaction_cursor, wallet_currency)
        .await?;

    let tx_1 = usd_transactions
        .next()
        .await
        .expect("Expected transaction, found None");

    assert_eq!(tx_1.cursor, "YXJyYXljb25uZWN0aW9uOjQxMA==");
    assert_eq!(tx_1.node.status, TxStatus::SUCCESS);
    assert_eq!(tx_1.node.settlement_amount, 0.00011503663416);
    assert_eq!(tx_1.node.settlement_currency, WalletCurrency::USD);
    assert_eq!(tx_1.node.memo, None);
    assert_eq!(tx_1.node.created_at, 1602800629);

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

/// Test to generate onchain deposit address
#[tokio::test]
async fn onchain_deposit_address() -> anyhow::Result<()> {
    let config = staging_wallet_configuration();
    let wallet_client = GaloyClient::connect(config).await?;
    let wallet_id: WalletId = "e705aa02-052b-4c3e-be2b-523c98a1aec4".to_string();

    let onchain_address = wallet_client.onchain_address(wallet_id).await?;
    println!("{:?}", onchain_address);
    assert!(onchain_address.address.len() == 42);
    Ok(())
}

/// Test making an onchain payment
#[tokio::test]
async fn onchain_payment() -> anyhow::Result<()> {
    let config = staging_wallet_configuration();
    let wallet_client = GaloyClient::connect(config).await?;

    let btc_wallet_id: WalletId = "e705aa02-052b-4c3e-be2b-523c98a1aec4".to_string();
    let onchain_address: OnChainAddress = "tb1qy4vzwfnfdsxmkjw8wh4mhw3h6gy7g2gw48zzkr".to_string();
    let memo = "".to_string();
    let amount = 1001;
    let target_conf = 2;

    let payment_result = wallet_client
        .send_onchain_payment(
            onchain_address,
            amount,
            Some(memo),
            target_conf,
            btc_wallet_id,
        )
        .await?;
    println!("{:?}", payment_result);

    assert_eq!(
        payment_result,
        stablesats_on_chain_payment::PaymentSendResult::SUCCESS
    );

    Ok(())
}
// /// Test to get onchain transaction fee
// #[tokio::test]
// async fn onchain_tx_fee() -> anyhow::Result<()> {
//     let config = staging_wallet_configuration();
//     let wallet_client = GaloyClient::connect(config).await?;

//     let onchain_tx_fee = wallet_client.onchain_tx_fee().await?;

//     println!("{:?}", onchain_tx_fee);
//     Ok(())
// }
