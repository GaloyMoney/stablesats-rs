use std::env;

use bitfinex_client::*;

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serial_test::serial;
use shared::exchanges_config::BitfinexConfig;

pub async fn configured_client() -> anyhow::Result<BitfinexClient> {
    let api_key = env::var("BITFINEX_API_KEY").expect("BITFINEX_API_KEY not set");
    let secret_key = env::var("BITFINEX_SECRET_KEY").expect("BITFINEX_SECRET_KEY not set");

    let client = BitfinexClient::new(BitfinexConfig {
        api_key,
        secret_key,
        simulated: true,
    })
    .await?;

    Ok(client)
}

#[tokio::test]
#[serial]
async fn trading() -> anyhow::Result<()> {
    let client = configured_client().await?;

    //
    // Close all positons
    //
    let positions = client.get_positions().await?;
    dbg!(positions.clone());

    let swap_position = positions
        .into_iter()
        .find(|position| position.symbol == Instrument::TestBtcUsdSwap.to_string());
    if let Some(swap_position) = swap_position {
        let submitted_order = client
            .submit_order(
                ClientId::new(),
                Decimal::NEGATIVE_ONE * swap_position.amount,
                swap_position.leverage,
            )
            .await?;
        dbg!(submitted_order);
    }
    // Verify
    let positions = client.get_positions().await?;
    dbg!(positions.clone());
    assert!(positions.is_empty());

    //
    // Move funds from trading to funding
    //
    let wallets = client.get_wallets().await?;
    dbg!(wallets.clone());
    let trading_wallet = wallets.into_iter().find(|wallet| {
        wallet.wallet_type == Wallet::MARGIN.to_string()
            && wallet.currency == Currency::TESTUSDTF0.to_string()
    });
    if let Some(trading_wallet) = trading_wallet {
        let details = client
            .transfer_trading_to_funding(ClientId::new(), trading_wallet.balance)
            .await?;
        dbg!(details);
    }
    // Verify
    let wallets = client.get_wallets().await?;
    dbg!(wallets.clone());
    let trading_wallet = wallets.into_iter().find(|wallet| {
        wallet.wallet_type == Wallet::MARGIN.to_string()
            && wallet.currency == Currency::TESTUSDTF0.to_string()
    });
    if let Some(trading_wallet) = trading_wallet {
        assert!(trading_wallet.balance.is_zero());
    }

    //
    // Move funds from funding to trading
    // ie. USDt from exchange to USDTF0
    //
    let collateral_amount = dec!(100);
    let details = client
        .transfer_funding_to_trading(ClientId::new(), collateral_amount)
        .await?;
    dbg!(details);
    // Verify
    let wallets = client.get_wallets().await?;
    dbg!(wallets.clone());
    let trading_wallet = wallets
        .into_iter()
        .find(|wallet| {
            wallet.wallet_type == Wallet::MARGIN.to_string()
                && wallet.currency == Currency::TESTUSDTF0.to_string()
        })
        .unwrap();
    assert_eq!(trading_wallet.balance, collateral_amount);

    //
    // Take a short positon
    //

    //
    // Close all positons
    //

    //
    // Move funds from trading to funding
    //

    Ok(())
}
