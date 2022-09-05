use std::time::Duration;

pub struct HedgingAppConfig {
    pub pg_con: String,
    pub migrate_on_start: bool,
    pub okex_poll_delay: Duration,
}
