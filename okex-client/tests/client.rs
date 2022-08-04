use okex_client::OkexClient;
use reqwest;

#[tokio::test]
async fn test_authenticate_with_okex() -> anyhow::Result<()> {
    // 1. Construct authentication header
    let config = ApiConfig {
        api_key: ApiKey::from(""),
        secret_key: SecretKey::from(""),
        passphrase: PassPhrase::from(""),
    };
    let okex_client = OkexClient::new(config);
    // 2. Make a request to a protected endpoint
    let pending_orders = "/api/v5/trade/orders-pending";
    let resp = okex_client.client.get(pending_orders).await?;

    Ok(())
}
