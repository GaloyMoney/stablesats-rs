use std::env;

use ::bria_client::*;

fn client_configuration() -> BriaClientConfig {
    let url = "http://localhost:2742".to_string();
    let profile_api_key = env::var("BRIA_KEY").expect("BRIA_KEY not set");
    let wallet_name = "default".to_string();
    let payout_queue_name = "default".to_string();
    let onchain_address_external_id = "stablesats_external_id".to_string();

    BriaClientConfig {
        url,
        profile_api_key,
        wallet_name,
        onchain_address_external_id,
        payout_queue_name,
    }
}

#[tokio::test]
async fn onchain_address() -> anyhow::Result<()> {
    let config = client_configuration();
    let mut client = BriaClient::connect(config).await?;

    let onchain_address = client.onchain_address().await?;
    assert_eq!(onchain_address.address.len(), 44);

    Ok(())
}

#[tokio::test]
async fn send_onchain_payment() -> anyhow::Result<()> {
    let config = client_configuration();
    let mut client = BriaClient::connect(config).await?;
    let destination = "bcrt1q5cwegu66cf344du3ffrvnwjz9u246xlydqezsa".to_string();
    let satoshis = rust_decimal::Decimal::from(5000);

    let id = client.send_onchain_payment(destination, satoshis).await?;
    assert!(!id.is_empty());

    Ok(())
}
