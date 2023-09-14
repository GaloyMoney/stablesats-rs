use chrono::Duration;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[serde_with::serde_as]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ExchangePriceCacheConfig {
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    #[serde(default = "default_stale_after_duration")]
    pub stale_after: Duration,
    #[serde(default)]
    pub dev_mock_price_btc_in_usd: Option<Decimal>,
}

fn default_stale_after_duration() -> Duration {
    Duration::from_std(std::time::Duration::from_secs(30)).unwrap()
}

impl Default for ExchangePriceCacheConfig {
    fn default() -> Self {
        ExchangePriceCacheConfig {
            stale_after: default_stale_after_duration(),
            dev_mock_price_btc_in_usd: None,
        }
    }
}
