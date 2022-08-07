use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Serialize, Deserialize, Debug)]
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
