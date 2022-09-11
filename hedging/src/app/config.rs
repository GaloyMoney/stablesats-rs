use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HedgingAppConfig {
    #[serde(default)]
    pub pg_con: String,
    #[serde(default = "bool_true")]
    pub migrate_on_start: bool,
    #[serde(default = "default_okex_poll_frequency")]
    pub okex_poll_frequency: Duration,
}

impl Default for HedgingAppConfig {
    fn default() -> Self {
        Self {
            pg_con: "".to_string(),
            migrate_on_start: true,
            okex_poll_frequency: default_okex_poll_frequency(),
        }
    }
}

fn bool_true() -> bool {
    true
}

fn default_okex_poll_frequency() -> Duration {
    Duration::from_secs(10)
}
