use galoy_wallet::*;
use std::env;

/// Test send an non-authenticated query to the Galoy Graphql API
#[tokio::test]
async fn get_btc_price() -> anyhow::Result<()> {
    let api = env::var("GALOY_GRAPH_URI").expect("GALOY_GRAPH_URI not set");
    let phone_number = env::var("WALLET_PHONE_NUMBER").expect("WALLET_PHONE_NUMBER not set");
    let config = GaloyClientConfig { api, phone_number };

    let wallet = GaloyClient::new(config);
    let price = wallet.btc_price().await?;

    println!("{:#?}", price);

    assert_eq!(price.offset, 12);
    assert_eq!(
        price.currency_unit,
        btc_price::ExchangeCurrencyUnit::USDCENT
    );

    Ok(())
}

/// Get an authentication code from the Galoy Graphql API
#[tokio::test]
async fn authentication_code() -> anyhow::Result<()> {
    let api = env::var("GALOY_GRAPH_URI").expect("GALOY_GRAPH_URI not set");
    let phone_number = env::var("WALLET_PHONE_NUMBER").expect("WALLET_PHONE_NUMBER not set");
    let config = GaloyClientConfig { api, phone_number };

    let wallet = GaloyClient::new(config);
    let auth_code_response = wallet.authentication_code().await?;

    assert_eq!(
        auth_code_response,
        auth_code::AuthCodeUserRequestAuthCode {
            success: Some(true),
            errors: vec![auth_code::AuthCodeUserRequestAuthCodeErrors {
                message: "".to_string()
            }]
        }
    );

    Ok(())
}
