use chrono::Duration;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use url::Url;

#[serde_with::serde_as]
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PriceFeedConfig {
    #[serde(default = "default_url")]
    pub url: Url,
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    #[serde(default = "default_throttle")]
    pub rate_limit_interval: Duration,
    #[serde(default)]
    pub dev_mock_price_btc_in_usd: Option<Decimal>,
}

impl Default for PriceFeedConfig {
    fn default() -> Self {
        Self {
            url: default_url(),
            rate_limit_interval: default_throttle(),
            dev_mock_price_btc_in_usd: None,
        }
    }
}

fn default_url() -> Url {
    Url::parse("wss://ws.okx.com:8443/ws/v5/public").unwrap()
}

fn default_throttle() -> Duration {
    Duration::from_std(std::time::Duration::from_secs(2)).unwrap()
}
