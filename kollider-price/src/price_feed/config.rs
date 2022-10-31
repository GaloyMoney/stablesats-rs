use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct KolliderPriceFeedConfig {
    pub url: Url,
}

impl Default for KolliderPriceFeedConfig {
    fn default() -> Self {
        Self {
            url: Url::parse("wss://testnet.kollider.xyz/v1/ws/").unwrap(),
        }
    }
}
