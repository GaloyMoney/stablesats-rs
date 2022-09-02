use std::env;

use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use okex_client::*;

async fn configured_okex_client() -> anyhow::Result<OkexClient> {
    let api_key = env::var("OKEX_API_KEY").expect("OKEX_API_KEY not set");
    let passphrase = env::var("OKEX_PASSPHRASE").expect("OKEX_PASS_PHRASE not set");
    let secret_key = env::var("OKEX_SECRET_KEY").expect("OKEX_SECRET_KEY not set");

    let client = OkexClient::new(OkexClientConfig {
        api_key,
        passphrase,
        secret_key,
        simulated: true,
    })
    .await?;

    Ok(client)
}

#[tokio::test]
async fn get_deposit_address_data() -> anyhow::Result<()> {
    let client = configured_okex_client().await?;
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
        simulated: true,
    })
    .await;

    assert!(client.is_err());

    if let Err(OkexClientError::UnexpectedResponse { msg, .. }) = client {
        assert!(msg.contains("header"));
    } else {
        assert!(false)
    }

    Ok(())
}

#[tokio::test]
async fn funding_account_balance() -> anyhow::Result<()> {
    let client = configured_okex_client().await?;
    let avail_balance = client.funding_account_balance().await?;
    let balance = avail_balance.amt_in_btc;
    let minimum_balance = dec!(0);
    assert!(balance >= minimum_balance);

    Ok(())
}

#[tokio::test]
async fn trading_account_balance() -> anyhow::Result<()> {
    let client = configured_okex_client().await?;
    let avail_balance = client.trading_account_balance().await?;
    let minimum_balance = dec!(0);

    assert!(avail_balance.amt_in_btc >= minimum_balance);

    Ok(())
}

#[tokio::test]
#[ignore = "only works against real okex client"]
async fn deposit_status() -> anyhow::Result<()> {
    if let (Ok(deposit_addr), Ok(deposit_amount)) = (
        env::var("OKEX_DEPOSIT_ADDRESS"),
        env::var("OKEX_DEPOSIT_AMOUNT"),
    ) {
        let amt = Decimal::from_str_exact(&deposit_amount)?;
        let client = configured_okex_client().await?;

        let deposit = client.fetch_deposit(deposit_addr, amt).await?;

        assert_eq!(deposit.status, "2".to_string());
    }
    Ok(())
}

#[tokio::test]
#[ignore = "only works against real okex client"]
async fn withdraw_to_onchain_address() -> anyhow::Result<()> {
    let amount = Decimal::from_str_exact(OKEX_MINIMUM_WITHDRAWAL_AMOUNT)?;
    let fee = Decimal::from_str_exact(OKEX_MINIMUM_WITHDRAWAL_FEE)?;
    if let Ok(onchain_address) = env::var("ONCHAIN_BTC_WITHDRAWAL_ADDRESS") {
        let client = configured_okex_client().await?;
        let withdraw_id = client
            .withdraw_btc_onchain(amount, fee, onchain_address)
            .await?;

        assert!(withdraw_id.value.len() == 8);
    }
    Ok(())
}

#[tokio::test]
#[ignore = "transfer call is rate limited"]
async fn transfer_trading_to_funding() -> anyhow::Result<()> {
    let client = configured_okex_client().await?;
    let amount = dec!(0.00001);
    let transfer_id = client.transfer_trading_to_funding(amount).await?;

    assert!(transfer_id.value.len() == 9);

    Ok(())
}

#[tokio::test]
#[ignore = "transfer call is rate limited"]
async fn transfer_state() -> anyhow::Result<()> {
    let client = configured_okex_client().await?;
    let amount = dec!(0.00001);
    let transfer_id = client.transfer_funding_to_trading(amount).await?;

    let transfer_state = client.transfer_state(transfer_id).await?;

    assert_eq!(transfer_state.value, "success".to_string());

    Ok(())
}

#[tokio::test]
async fn open_close_position() -> anyhow::Result<()> {
    let client = configured_okex_client().await?;

    client.close_positions().await?;

    client
        .place_order(OkexOrderSide::Sell, BtcUsdSwapContracts(1))
        .await?;

    let position = client.get_position_in_usd().await?;

    assert!(position.value < dec!(-95));
    assert!(position.value > dec!(-105));

    assert!(client.close_positions().await.is_ok());

    Ok(())
}
