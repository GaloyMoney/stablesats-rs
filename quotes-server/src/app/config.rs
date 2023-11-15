use chrono::Duration;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub struct ExchangeWeights {
    pub okex: Option<Decimal>,
    pub bitfinex: Option<Decimal>,
}

#[serde_with::serde_as]
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct QuotesServerHealthCheckConfig {
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    #[serde(default = "default_unhealthy_msg_interval_price")]
    pub unhealthy_msg_interval_price: Duration,
}

impl Default for QuotesServerHealthCheckConfig {
    fn default() -> Self {
        Self {
            unhealthy_msg_interval_price: default_unhealthy_msg_interval_price(),
        }
    }
}

fn default_unhealthy_msg_interval_price() -> Duration {
    Duration::from_std(std::time::Duration::from_secs(20))
        .expect("bad default unhealthy_after_msg_delay")
}

#[serde_with::serde_as]
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct QuotesConfig {
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    #[serde(default = "default_expiration_interval")]
    pub expiration_interval: Duration,
}

impl Default for QuotesConfig {
    fn default() -> Self {
        Self {
            expiration_interval: default_expiration_interval(),
        }
    }
}

fn default_expiration_interval() -> Duration {
    Duration::from_std(std::time::Duration::from_secs(120)) // 2 minutes = 120 seconds
        .expect("bad default expiration_interval")
}
