use std::env;

use okex_client::{OkexClient, OkexClientConfig, OkexClientError};

#[tokio::test]
async fn get_deposit_address_data() -> anyhow::Result<()> {
    let api_key = env::var("OKEX_API_KEY").expect("OKEX_API_KEY not set");
    let pass_phrase = env::var("OKEX_PASS_PHRASE").expect("OKEX_PASS_PHRASE not set");
    let secret_key = env::var("OKEX_SECRET_KEY").expect("OKEX_SECRET_KEY not set");
    let client = OkexClient::new(OkexClientConfig {
        api_key,
        pass_phrase,
        secret_key,
    });
    let address = client.get_funding_deposit_address().await?;
    assert!(address.value.len() > 10);

    Ok(())
}

#[tokio::test]
async fn client_is_missing_header() -> anyhow::Result<()> {
    let client = OkexClient::new(OkexClientConfig {
        api_key: "".to_string(),
        pass_phrase: "".to_string(),
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
