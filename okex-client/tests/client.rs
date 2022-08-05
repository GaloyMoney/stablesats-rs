use okex_client::OkexClient;
use reqwest;

#[tokio::test]
async fn test_authenticate_with_okex() -> anyhow::Result<()> {
    let config = ApiConfig {
        api_key: Some(std::env::var("API_KEY").unwrap_or("apikey".to_string())),
        secret_key: Some(std::env::var("SECRET_KEY").unwrap_or("secretkey".to_string())),
        passphrase: Some(
            std::env::var("PASSPHRASE").unwrap_or("passphrase".to_string()),
        ),
    };
    let okex_client = OkexClient::new(config);
    let pending_orders = "/api/v5/trade/orders-pending";
    let resp = okex_client.client.get(pending_orders).await?.text().await?;

    assert_eq!(resp, "received");

    Ok(())
}
