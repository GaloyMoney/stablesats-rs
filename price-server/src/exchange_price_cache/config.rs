use serde::{Deserialize, Serialize};
use std::time::Duration;

#[serde_with::serde_as]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ExchangePriceCacheConfig {
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    #[serde(default = "default_stale_after_duration")]
    pub stale_after: Duration,
}

fn default_stale_after_duration() -> Duration {
    Duration::from_secs(30)
}

impl Default for ExchangePriceCacheConfig {
    fn default() -> Self {
        ExchangePriceCacheConfig {
            stale_after: default_stale_after_duration(),
        }
    }
}
