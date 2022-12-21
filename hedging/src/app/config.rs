use serde::{Deserialize, Serialize};
use std::time::Duration;

#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HedgingAppConfig {
    #[serde(default)]
    pub pg_con: String,
    #[serde(default = "bool_true")]
    pub migrate_on_start: bool,
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    #[serde(default = "default_okex_poll_frequency")]
    pub okex_poll_frequency: Duration,
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    #[serde(default = "default_unhealthy_msg_interval")]
    pub unhealthy_msg_interval_liability: chrono::Duration,
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    #[serde(default = "default_unhealthy_msg_interval")]
    pub unhealthy_msg_interval_position: chrono::Duration,
}

impl Default for HedgingAppConfig {
    fn default() -> Self {
        Self {
            pg_con: "".to_string(),
            migrate_on_start: true,
            okex_poll_frequency: default_okex_poll_frequency(),
            unhealthy_msg_interval_liability: default_unhealthy_msg_interval(),
            unhealthy_msg_interval_position: default_unhealthy_msg_interval(),
        }
    }
}

fn bool_true() -> bool {
    true
}

fn default_okex_poll_frequency() -> Duration {
    Duration::from_secs(10)
}

fn default_unhealthy_msg_interval() -> chrono::Duration {
    chrono::Duration::from_std(Duration::from_secs(20))
        .expect("bad default unhealthy_after_msg_delay")
}
