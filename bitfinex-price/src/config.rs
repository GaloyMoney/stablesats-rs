use serde::{Deserialize, Serialize};
use url::Url;

pub const BTC_USD_SWAP: &str = "tBTCUSD";

#[serde_with::serde_as]
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PriceFeedConfig {
    #[serde(default = "default_url")]
    pub url: Url,
}

impl Default for PriceFeedConfig {
    fn default() -> Self {
        Self { url: default_url() }
    }
}

fn default_url() -> Url {
    Url::parse("wss://api-pub.bitfinex.com/ws/2").unwrap()
}
