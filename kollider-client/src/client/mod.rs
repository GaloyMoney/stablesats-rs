use reqwest::Client;

use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct KolliderClientConfig {
    pub url: String,
    pub api_key: String,
    pub passphrase: String,
    pub secret: String,
}

#[derive(Clone)]
pub struct KolliderClient<'a> {
    client: Client,
    api_key: &'a str,
    passphrase: &'a str,
    secret: &'a str,
    base_url: &'a str,
}

impl<'a> KolliderClient<'a> {
    pub fn new(
        base_url_param: &'a str,
        apikey_param: &'a str,
        passphrase_param: &'a str,
        secret_param: &'a str,
    ) -> KolliderClient<'a> {
        KolliderClient {
            client: reqwest::Client::new(),
            base_url: base_url_param,
            api_key: apikey_param,
            passphrase: passphrase_param,
            secret: secret_param,
        }
    }

    pub async fn get_products(&self) -> Result<String, String> {
        let path = "/market/products";
        let res = self
            .client
            .get(format!("{}{}", self.base_url, path))
            .send()
            .await
            .unwrap();

        Ok(res.text().await.unwrap())
    }
}
