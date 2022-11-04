#[cfg(test)]
use kollider_client::KolliderClientConfig;
use kollider_client::{KolliderClient, KolliderOrderSide};

use std::fs;

fn create_test_client() -> anyhow::Result<KolliderClient> {
    let content = fs::read_to_string("config.json")?;
    let config = serde_json::from_str::<KolliderClientConfig>(&content)?;
    Ok(KolliderClient::new(config))
}

#[tokio::test]
async fn get_products() -> anyhow::Result<()> {
    if let Ok(client) = create_test_client() {
        let products = client.get_products().await?;
        assert_eq!("BTCUSD.PERP", products.btcusd_perp.symbol);
        println!("products: {:?}", products.btcusd_perp);
    }
    Ok(())
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
        let order = client.place_order(KolliderOrderSide::Ask, 10, 100).await?;
        assert_eq!(10, order.quantity);
        println!("order: {:?}", order);
    }
    Ok(())
}

#[tokio::test]
async fn get_open_positions() -> anyhow::Result<()> {
    if let Ok(client) = create_test_client() {
        let pos = client.get_open_positions().await?;
        println!("order: {:?}", pos);
    }
    Ok(())
}
