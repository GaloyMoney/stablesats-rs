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
        assert_eq!("BTCUSD.PERP", products.btcusd_perp.symbol);
        println!("products: {:?}", products.btcusd_perp);
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

#[tokio::test]
async fn place_order() -> anyhow::Result<()> {
    if let Ok(client) = create_test_client() {
        let order = client.place_order(10, 1200).await?;
        println!("order: {:?}", order);
    }
    Ok(())
}
