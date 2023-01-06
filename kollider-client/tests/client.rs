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
    let conf = KolliderClientConfig {
        url: "https://testnet.kollider.xyz/v1".to_string(),
        ..KolliderClientConfig::default()
    };
    let client = KolliderClient::new(conf);
    let products = client.get_products().await?;
    assert_eq!("BTCUSD.PERP", products.btcusd_perp.symbol);
    Ok(())
}

#[tokio::test]
#[ignore]
async fn get_user_balances() -> anyhow::Result<()> {
    let client = create_test_client().expect("Could not create test-client");
    let balance = client.get_user_balances().await?;
    dbg!(balance);
    Ok(())
}

#[tokio::test]
#[ignore]
async fn place_order() -> anyhow::Result<()> {
    let client = create_test_client().expect("Could not create test-client");
    let order = client.place_order(KolliderOrderSide::Sell, 10, 700).await?;
    let open_orders = client.get_open_orders().await?;
    assert_eq!(10, order.quantity);
    dbg!("order: {} ", open_orders);
    Ok(())
}

#[tokio::test]
#[ignore]
async fn get_open_positions() -> anyhow::Result<()> {
    let client = create_test_client().expect("Could not create test-client");
    let pos = client.get_open_positions().await?;
    dbg!(pos);
    Ok(())
}
