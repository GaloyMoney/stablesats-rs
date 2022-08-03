use anyhow::Context;
use serde::Deserialize;
use std::path::Path;

use shared::pubsub::PubSubConfig;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub pubsub: PubSubConfig,
}

impl Config {
    pub fn from_path(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let config_file = std::fs::read_to_string(path).context("Couldn't read config file")?;
        let config: Config =
            serde_yaml::from_str(&config_file).context("Couldn't parse config file")?;
        Ok(config)
    }
}
