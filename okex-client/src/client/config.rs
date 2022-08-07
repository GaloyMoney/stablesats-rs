use anyhow::{self, Context};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub api_key: String,
    pub secret_key: String,
    pub pass_phrase: String,
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVar {
    pub okex_client: ApiConfig,
}

pub struct EnvOverride {
    pub api_key: Option<String>,
    pub secret_key: Option<String>,
    pub pass_phrase: Option<String>,
    pub base_url: Option<String>,
}

impl EnvVar {
    pub fn from_path(
        path: impl AsRef<Path>,
        EnvOverride {
            api_key,
            secret_key,
            pass_phrase,
            base_url,
        }: EnvOverride,
    ) -> anyhow::Result<Self> {
        let config_file =
            std::fs::read_to_string(path).context("Couldn't read configuration file")?;
        let mut config: EnvVar =
            serde_yaml::from_str(&config_file).context("Couldn't parse config file")?;
        if let (Some(api_key), Some(secret_key), Some(pass_phrase), Some(base_url)) =
            (api_key, secret_key, pass_phrase, base_url)
        {
            config.okex_client.api_key = api_key;
            config.okex_client.secret_key = secret_key;
            config.okex_client.pass_phrase = pass_phrase;
            config.okex_client.base_url = base_url;
        }

        Ok(config)
    }
}
