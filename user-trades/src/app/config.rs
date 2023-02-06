use serde::{Deserialize, Serialize};
use std::time::Duration;

#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTradesConfig {
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    #[serde(default = "default_galoy_poll_frequency")]
    pub galoy_poll_frequency: Duration,
}

impl Default for UserTradesConfig {
    fn default() -> Self {
        Self {
            galoy_poll_frequency: default_galoy_poll_frequency(),
        }
    }
}

fn default_galoy_poll_frequency() -> Duration {
    Duration::from_secs(10)
}
