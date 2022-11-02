mod client;
pub use client::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn load_config() -> Option<KolliderClientConfig> {
        let content = fs::read_to_string("config.json").unwrap();
        Some(serde_json::from_str::<KolliderClientConfig>(&content).unwrap())
    }

    #[tokio::test]
    async fn it_works() {
        if let Some(config) = load_config() {
            let client = KolliderClient::new(
                &config.url,
                &config.api_key,
                &config.passphrase,
                &config.secret,
            );
            let products = client.get_products().await.unwrap();
            println!("products: {}", products);
        }
    }
}
