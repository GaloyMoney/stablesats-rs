use std::env;

use rust_decimal::Decimal;

use okex_client::{
    OkexClient, OkexClientConfig, OkexClientError, OKEX_MINIMUM_WITHDRAWAL_AMOUNT,
    OKEX_MINIMUM_WITHDRAWAL_FEE,
};

fn configured_okex_client() -> OkexClient {
    let api_key = env::var("OKEX_API_KEY").expect("OKEX_API_KEY not set");
    let passphrase = env::var("OKEX_PASSPHRASE").expect("OKEX_PASS_PHRASE not set");
    let secret_key = env::var("OKEX_SECRET_KEY").expect("OKEX_SECRET_KEY not set");
    OkexClient::new(OkexClientConfig {
        api_key,
        passphrase,
        secret_key,
    })
}

#[tokio::test]
async fn get_deposit_address_data() -> anyhow::Result<()> {
    let address = configured_okex_client()
        .get_funding_deposit_address()
        .await?;
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
async fn funding_account_balance() -> anyhow::Result<()> {
    let avail_balance = configured_okex_client().funding_account_balance().await?;
    let balance = avail_balance.amt_in_btc;
    let minimum_balance = Decimal::new(0, 2);
    assert!(balance >= minimum_balance);

    Ok(())
}

#[tokio::test]
async fn trading_account_balance() -> anyhow::Result<()> {
    let avail_balance = configured_okex_client().trading_account_balance().await?;
    let minimum_balance = Decimal::new(0, 2);
    assert!(avail_balance.amt_in_btc >= minimum_balance);

    Ok(())
}

#[tokio::test]
async fn deposit_status() -> anyhow::Result<()> {
    if let (Ok(deposit_addr), Ok(deposit_amount)) = (
        env::var("OKEX_DEPOSIT_ADDRESS"),
        env::var("OKEX_DEPOSIT_AMOUNT"),
    ) {
        let amt = Decimal::from_str_exact(&deposit_amount)?;

        let deposit = configured_okex_client()
            .fetch_deposit(deposit_addr, amt)
            .await?;

        assert_eq!(deposit.status, "2".to_string());
    }
    Ok(())
}

#[tokio::test]
async fn withdraw_to_onchain_address() -> anyhow::Result<()> {
    let amount = Decimal::from_str_exact(OKEX_MINIMUM_WITHDRAWAL_AMOUNT)?;
    let fee = Decimal::from_str_exact(OKEX_MINIMUM_WITHDRAWAL_FEE)?;
    if let Ok(onchain_address) = env::var("ONCHAIN_BTC_WITHDRAWAL_ADDRESS") {
        let withdraw_id = configured_okex_client()
            .withdraw_btc_onchain(amount, fee, onchain_address)
            .await?;

        assert!(withdraw_id.value.len() == 8);
    }
    Ok(())
}

#[tokio::test]
#[ignore = "transfer call is rate limited"]
async fn transfer_trading_to_funding() -> anyhow::Result<()> {
    let amount = Decimal::new(1, 5);
    let transfer_id = configured_okex_client()
        .transfer_trading_to_funding(amount)
        .await?;

    assert!(transfer_id.value.len() == 9);

    Ok(())
}

#[tokio::test]
#[ignore = "transfer call is rate limited"]
async fn transfer_funding_to_trading() -> anyhow::Result<()> {
    let amount = Decimal::new(1, 5);
    let transfer_id = configured_okex_client()
        .transfer_funding_to_trading(amount)
        .await?;

    assert!(transfer_id.value.len() == 9);

    Ok(())
}

#[tokio::test]
#[ignore = "transfer call is rate limited"]
async fn transfer_state() -> anyhow::Result<()> {
    let client = configured_okex_client();
    let amount = Decimal::new(1, 5);
    let transfer_id = client.transfer_funding_to_trading(amount).await?;

    let transfer_state = client.transfer_state(transfer_id).await?;

    assert_eq!(transfer_state.value, "success".to_string());

    Ok(())
}
