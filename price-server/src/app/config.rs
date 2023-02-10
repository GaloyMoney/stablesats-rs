use chrono::Duration;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub struct ExchangeWeights {
    pub okex: Option<Decimal>,
    pub bitfinex: Option<Decimal>,
    pub kollider: Option<Decimal>,
}

#[serde_with::serde_as]
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PriceServerHealthCheckConfig {
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    #[serde(default = "default_unhealthy_msg_interval_price")]
    pub unhealthy_msg_interval_price: Duration,
}

impl Default for PriceServerHealthCheckConfig {
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
