use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTradesConfig {
    #[serde(default)]
    pub pg_con: String,
    #[serde(default = "bool_true")]
    pub migrate_on_start: bool,
    #[serde(default = "default_balance_publish_frequency")]
    pub balance_publish_frequency: Duration,
    #[serde(default = "default_galoy_poll_frequency")]
    pub galoy_poll_frequency: Duration,
}

impl Default for UserTradesConfig {
    fn default() -> Self {
        Self {
            pg_con: "".to_string(),
            migrate_on_start: true,
            balance_publish_frequency: default_balance_publish_frequency(),
            galoy_poll_frequency: default_galoy_poll_frequency(),
        }
    }
}

fn default_balance_publish_frequency() -> Duration {
    Duration::from_secs(2)
}

fn default_galoy_poll_frequency() -> Duration {
    Duration::from_secs(5)
}

fn bool_true() -> bool {
    true
}
