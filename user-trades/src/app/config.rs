use std::time::Duration;

pub struct UserTradesAppConfig {
    pub pg_con: String,
    pub migrate_on_start: bool,
    pub publish_frequency: Duration,
    pub galoy_poll_frequency: Duration,
}
