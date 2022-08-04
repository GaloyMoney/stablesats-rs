use anyhow::Context;
use serde::Deserialize;
use std::path::Path;

use okex_price::PriceFeedConfig;
use price_server::PriceServerConfig;
use shared::pubsub::PubSubConfig;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub pubsub: PubSubConfig,
    #[serde(default)]
    pub price_server: PriceServerWrapper,
    #[serde(default)]
    pub okex_price_feed: PriceFeedConfigWrapper,
}

impl Config {
    pub fn from_path(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let config_file = std::fs::read_to_string(path).context("Couldn't read config file")?;
        let config: Config =
            serde_yaml::from_str(&config_file).context("Couldn't parse config file")?;
        Ok(config)
    }
}

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
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
