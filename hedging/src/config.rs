use rust_decimal::Decimal;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::time::Duration;

use crate::okex::OkexConfig;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExchangesConfig {
    pub okex: Option<ExchangeConfig<OkexConfig>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExchangeConfig<T: DeserializeOwned + Serialize + Default> {
    pub weight: Decimal,
    #[serde(bound = "T: DeserializeOwned")]
    #[serde(default)]
    pub config: T,
}

#[serde_with::serde_as]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HedgingAppConfig {
    #[serde(default)]
    pub health: HedgingAppHealthConfig,
}

#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HedgingAppHealthConfig {
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    #[serde(default = "default_unhealthy_msg_interval")]
    pub unhealthy_msg_interval_liability: chrono::Duration,
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    #[serde(default = "default_unhealthy_msg_interval")]
    pub unhealthy_msg_interval_position: chrono::Duration,
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    #[serde(default = "default_unhealthy_msg_interval")]
    pub unhealthy_msg_interval_price: chrono::Duration,
}

impl Default for HedgingAppHealthConfig {
    fn default() -> Self {
        Self {
            unhealthy_msg_interval_liability: default_unhealthy_msg_interval(),
            unhealthy_msg_interval_position: default_unhealthy_msg_interval(),
            unhealthy_msg_interval_price: default_unhealthy_msg_interval(),
        }
    }
}

fn default_unhealthy_msg_interval() -> chrono::Duration {
    chrono::Duration::from_std(Duration::from_secs(20))
        .expect("bad default unhealthy_after_msg_delay")
}
