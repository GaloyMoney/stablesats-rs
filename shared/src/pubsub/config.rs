use fred::prelude::*;
use serde::{Deserialize, Serialize};

use std::time::Duration;

#[serde_with::serde_as]
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PubSubConfig {
    pub host: Option<String>,
    #[serde(default = "default_port")]
    pub port: u16,
    pub password: Option<String>,
    pub sentinel: Option<SentinelConfig>,
    #[serde_as(as = "serde_with::DurationSeconds<u64>")]
    #[serde(default = "default_throttle")]
    pub rate_limit_interval: Duration,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SentinelConfig {
    pub hosts: Vec<HostPortTuple>,
    #[serde(default = "default_service_name")]
    pub service_name: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct HostPortTuple {
    pub host: String,
    #[serde(default = "default_sentinel_port")]
    pub port: u16,
}

impl Default for PubSubConfig {
    fn default() -> Self {
        Self {
            host: Some("localhost".to_string()),
            port: default_port(),
            password: None,
            sentinel: None,
            rate_limit_interval: default_throttle(),
        }
    }
}

fn default_port() -> u16 {
    6379
}

fn default_sentinel_port() -> u16 {
    26379
}

fn default_throttle() -> Duration {
    Duration::from_secs(2)
}

fn default_service_name() -> String {
    "mymaster".to_string()
}

impl From<PubSubConfig> for RedisConfig {
    fn from(config: PubSubConfig) -> Self {
        let mut ret = RedisConfig::default();
        if let Some(password) = config.password {
            ret.password = Some(password)
        }
        if let Some(host) = config.host {
            ret.server = ServerConfig::new_centralized(host, config.port)
        }
        if let Some(sentinel) = config.sentinel {
            ret.server = ServerConfig::new_sentinel(
                sentinel
                    .hosts
                    .into_iter()
                    .map(|h| (h.host, h.port))
                    .collect(),
                sentinel.service_name,
            )
        }
        ret
    }
}
