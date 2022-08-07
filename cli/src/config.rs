use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::path::Path;

use okex_price::PriceFeedConfig;
use price_server::PriceServerConfig;
use shared::pubsub::PubSubConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub pubsub: PubSubConfig,
    #[serde(default)]
    pub price_server: PriceServerWrapper,
    #[serde(default)]
    pub okex_price_feed: PriceFeedConfigWrapper,
}

pub struct EnvOverride {
    pub redis_password: Option<String>,
}

impl Config {
    pub fn from_path(
        path: impl AsRef<Path>,
        EnvOverride { redis_password }: EnvOverride,
    ) -> anyhow::Result<Self> {
        let config_file = std::fs::read_to_string(path).context("Couldn't read config file")?;
        let mut config: Config =
            serde_yaml::from_str(&config_file).context("Couldn't parse config file")?;
        if let Some(redis_password) = redis_password {
            config.pubsub.password = Some(redis_password);
        }
        Ok(config)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceServerWrapper {
    pub enabled: bool,
    #[serde(default)]
    pub config: PriceServerConfig,
}
impl Default for PriceServerWrapper {
    fn default() -> Self {
        Self {
            enabled: true,
            config: PriceServerConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceFeedConfigWrapper {
    pub enabled: bool,
    #[serde(default)]
    pub config: PriceFeedConfig,
}
impl Default for PriceFeedConfigWrapper {
    fn default() -> Self {
        Self {
            enabled: true,
            config: PriceFeedConfig::default(),
        }
    }
}
