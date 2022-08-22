use std::env;

use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use okex_client::*;

async fn configured_okex_client() -> anyhow::Result<OkexClient> {
    let api_key = env::var("OKEX_API_KEY").expect("OKEX_API_KEY not set");
    let passphrase = env::var("OKEX_PASSPHRASE").expect("OKEX_PASS_PHRASE not set");
    let secret_key = env::var("OKEX_SECRET_KEY").expect("OKEX_SECRET_KEY not set");
    let simulated = false;
    let position_mode = OkexPositionMode::Net;
    let position_side = OkexPositionSide::Net;
    let order_type = OkexOrderType::Market;
    let margin_mode = OkexMarginMode::Cross;

    let client = OkexClient::create(OkexClientConfig {
        api_key,
        passphrase,
        secret_key,
        simulated,
        position_mode,
        position_side,
        order_type,
        margin_mode,
    })
    .await?;

    Ok(client)
}

async fn demo_okex_client() -> anyhow::Result<Option<OkexClient>> {
    let api_key = env::var("OKEX_DEMO_API_KEY");
    let passphrase = env::var("OKEX_DEMO_PASSPHRASE");
    let secret_key = env::var("OKEX_DEMO_SECRET_KEY");
    let simulated = true;
    let position_mode = OkexPositionMode::Net;
    let position_side = OkexPositionSide::Net;
    let order_type = OkexOrderType::Market;
    let margin_mode = OkexMarginMode::Cross;

    if let (Ok(api_key), Ok(passphrase), Ok(secret_key)) = (api_key, passphrase, secret_key) {
        let client = OkexClient::create(OkexClientConfig {
            api_key,
            passphrase,
            secret_key,
            simulated,
            position_mode,
            position_side,
            order_type,
            margin_mode,
        })
        .await?;

        return Ok(Some(client));
    }

    Ok(None)
}

#[tokio::test]
async fn create_okex_client() -> anyhow::Result<()> {
    let client = demo_okex_client().await;

    assert!(client.is_ok());

    Ok(())
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
    let client = OkexClient::create(OkexClientConfig {
        api_key: "".to_string(),
        passphrase: "".to_string(),
        secret_key: "".to_string(),
        simulated: true,
        position_mode: OkexPositionMode::Net,
        position_side: OkexPositionSide::Net,
        order_type: OkexOrderType::Market,
        margin_mode: OkexMarginMode::Cross,
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
    if let Some(client) = demo_okex_client().await? {
        let avail_balance = client.funding_account_balance().await?;
        let balance = avail_balance.amt_in_btc;
        let minimum_balance = dec!(0);
        assert!(balance >= minimum_balance);
    }

    Ok(())
}

#[tokio::test]
async fn trading_account_balance() -> anyhow::Result<()> {
    if let Some(client) = demo_okex_client().await? {
        let avail_balance = client.trading_account_balance().await?;
        let minimum_balance = dec!(0);
        assert!(avail_balance.amt_in_btc >= minimum_balance);
    }

    Ok(())
}

#[tokio::test]
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
async fn withdraw_to_onchain_address() -> anyhow::Result<()> {
    let amount = Decimal::from_str_exact(OKEX_MINIMUM_WITHDRAWAL_AMOUNT)?;
    let fee = Decimal::from_str_exact(OKEX_MINIMUM_WITHDRAWAL_FEE)?;
    if let Ok(onchain_address) = env::var("ONCHAIN_BTC_WITHDRAWAL_ADDRESS") {
        if let Some(client) = demo_okex_client().await? {
            let withdraw_id = client
                .withdraw_btc_onchain(amount, fee, onchain_address)
                .await?;

            assert!(withdraw_id.value.len() == 8);
        }
    }
    Ok(())
}

#[tokio::test]
#[ignore = "transfer call is rate limited"]
async fn transfer_trading_to_funding() -> anyhow::Result<()> {
    if let Some(client) = demo_okex_client().await? {
        let amount = dec!(0.00001);
        let transfer_id = client.transfer_trading_to_funding(amount).await?;

        assert!(transfer_id.value.len() == 9);
    }

    Ok(())
}

#[tokio::test]
#[ignore = "transfer call is rate limited"]
async fn transfer_funding_to_trading() -> anyhow::Result<()> {
    if let Some(client) = demo_okex_client().await? {
        let amount = dec!(0.00001);
        let transfer_id = client.transfer_funding_to_trading(amount).await?;

        assert!(transfer_id.value.len() == 9);
    }

    Ok(())
}

#[tokio::test]
#[ignore = "transfer call is rate limited"]
async fn transfer_state() -> anyhow::Result<()> {
    if let Some(client) = demo_okex_client().await? {
        let amount = dec!(0.00001);
        let transfer_id = client.transfer_funding_to_trading(amount).await?;

        let transfer_state = client.transfer_state(transfer_id).await?;

        assert_eq!(transfer_state.value, "success".to_string());
    }

    Ok(())
}

#[tokio::test]
async fn place_order() -> anyhow::Result<()> {
    if let Some(client) = demo_okex_client().await? {
        let instrument = OkexInstrumentId::BtcUsdSwap;
        let order_side = OkexOrderSide::Buy;

        let order_id = client.place_order(instrument, order_side, 1).await?;

        assert!(order_id.value.len() == 18);
    }

    Ok(())
}

#[tokio::test]
async fn get_positions() -> anyhow::Result<()> {
    if let Some(client) = demo_okex_client().await? {
        let position = client.get_position().await?;

        assert_eq!(position.value.len(), 18);
    }

    Ok(())
}

#[tokio::test]
async fn close_positions() -> anyhow::Result<()> {
    if let Some(client) = demo_okex_client().await? {
        let instrument = OkexInstrumentId::BtcUsdSwap;
        let order_side = OkexOrderSide::Buy;
        let currency = "BTC".to_string();

        // 1. Open position
        client
            .place_order(instrument.clone(), order_side, 1)
            .await?;

        // 2. Close position(s)
        let position = client.close_positions(instrument, currency, false).await?;

        assert_eq!(position.inst_id, "BTC-USD-SWAP".to_string());
        assert_eq!(position.pos_side, "net".to_string());
    }

    Ok(())
}
