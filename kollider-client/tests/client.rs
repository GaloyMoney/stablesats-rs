use kollider_client::KolliderClient;
#[cfg(test)]
use kollider_client::KolliderClientConfig;

use std::fs;

fn create_test_client() -> anyhow::Result<KolliderClient> {
    let content = fs::read_to_string("config.json")?;
    let config = serde_json::from_str::<KolliderClientConfig>(&content)?;
    Ok(KolliderClient::new(config))
}

#[tokio::test]
async fn get_products() {
    if let Ok(client) = create_test_client() {
        let products = client.get_products().await.unwrap();
        println!("products: {}", products);
    }
}

#[tokio::test]
async fn get_user_balances() -> anyhow::Result<()> {
    if let Ok(client) = create_test_client() {
        let balance = client.get_user_balances().await?;
        println!("balance: {:?}", balance);
    }
    Ok(())
}
