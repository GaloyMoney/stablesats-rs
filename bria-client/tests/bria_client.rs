use std::env;

use ::bria_client::*;

fn client_configuration() -> BriaClientConfig {
    let url = env::var("BRIA_URL").expect("BRIA_URL not set");
    let key = env::var("BRIA_KEY").expect("BRIA_KEY not set");
    let wallet_name = env::var("BRIA_WALLET_NAME").expect("BRIA_WALLET_NAME not set");
    let external_id = env::var("BRIA_EXTERNAL_ID").expect("BRIA_EXTERNAL_ID not set");
    let payout_queue_name =
        env::var("BRIA_PAYOUT_QUEUE_NAME").expect("BRIA_PAYOUT_QUEUE_NAME not set");

    BriaClientConfig {
        url,
        key,
        wallet_name,
        external_id,
        payout_queue_name,
    }
}

#[tokio::test]
async fn new_address() -> anyhow::Result<()> {
    let config = client_configuration();
    let client = BriaClient::new(config);

    let address = client.get_address().await?;
    assert_eq!(address.address.len(), 44);

    Ok(())
}
