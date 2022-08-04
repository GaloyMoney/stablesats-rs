use serde::Deserialize;
use url::Url;

#[derive(Clone, Deserialize, Debug)]
pub struct PriceFeedConfig {
    pub url: Url,
}

impl Default for PriceFeedConfig {
    fn default() -> Self {
        Self {
            url: Url::parse("wss://ws.okx.com:8443/ws/v5/public").unwrap(),
        }
    }
}
