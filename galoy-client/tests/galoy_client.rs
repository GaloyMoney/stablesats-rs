use std::env;

use galoy_client::*;

async fn configured_client() -> anyhow::Result<GaloyClient> {
    let api = env::var("GALOY_GRAPHQL_URI")?;
    let phone_number = env::var("GALOY_PHONE_NUMBER")?;
    let code = env::var("GALOY_PHONE_CODE")?;

    let client = GaloyClient::connect(GaloyClientConfig {
        api,
        phone_number,
        auth_code: code,
    })
    .await?;

    Ok(client)
}

/// Test to get transactions list of the default wallet
#[tokio::test]
async fn transactions_list() -> anyhow::Result<()> {
    if let Ok(client) = configured_client().await {
        let transactions = client.transactions_list(None).await?;
        assert!(transactions.list.len() > 0);
    }
    Ok(())
}
