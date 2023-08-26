use std::env;

use ::bria_client::*;

fn client_configuration() -> BriaClientConfig {
    let url = "http://localhost:2742".to_string();
    let key = env::var("BRIA_KEY").expect("BRIA_KEY not set");
    let wallet_name = "default".to_string();
    let payout_queue_name = "default".to_string();
    let external_id = "stablesats_external_id".to_string();

    BriaClientConfig {
        url,
        key,
        wallet_name,
        external_id,
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
