use anyhow::Context;
use kollider_price::config::KolliderPriceFeedConfig;
use serde::{Deserialize, Serialize};
use std::path::Path;

use galoy_client::GaloyClientConfig;
use hedging::HedgingAppConfig;
use okex_client::OkexClientConfig;
use okex_price::PriceFeedConfig;
use price_server::{
    ExchangePriceCacheConfig, FeeCalculatorConfig, PriceServerConfig, PriceServerHealthCheckConfig,
};
use shared::pubsub::PubSubConfig;
use user_trades::UserTradesConfig;

use super::tracing::TracingConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub pubsub: PubSubConfig,
    #[serde(default)]
    pub tracing: TracingConfig,
    #[serde(default)]
    pub price_server: PriceServerWrapper,
    #[serde(default)]
    pub okex_price_feed: PriceFeedConfigWrapper,
    #[serde(default)]
    pub user_trades: UserTradesConfigWrapper,
    #[serde(default)]
    pub galoy: GaloyClientConfig,
    #[serde(default)]
    pub okex: OkexClientConfig,
    #[serde(default)]
    pub hedging: HedgingConfigWrapper,
    #[serde(default)]
    pub kollider_price_feed: KolliderPriceFeedConfigWrapper,
}

pub struct EnvOverride {
    pub redis_password: Option<String>,
    pub user_trades_pg_con: String,
    pub hedging_pg_con: String,
    pub okex_secret_key: String,
    pub okex_passphrase: String,
    pub galoy_phone_code: String,
}

impl Config {
    pub fn from_path(
        path: impl AsRef<Path>,
        EnvOverride {
            redis_password,
            user_trades_pg_con,
            galoy_phone_code,
            okex_passphrase,
            okex_secret_key,
            hedging_pg_con,
        }: EnvOverride,
    ) -> anyhow::Result<Self> {
        let config_file = std::fs::read_to_string(path).context("Couldn't read config file")?;
        let mut config: Config =
            serde_yaml::from_str(&config_file).context("Couldn't parse config file")?;
        if let Some(redis_password) = redis_password {
            config.pubsub.password = Some(redis_password);
        }

        config.user_trades.config.pg_con = user_trades_pg_con;
        config.galoy.auth_code = galoy_phone_code;
        config.hedging.config.pg_con = hedging_pg_con;
        config.okex.secret_key = okex_secret_key;
        config.okex.passphrase = okex_passphrase;

        Ok(config)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceServerWrapper {
    #[serde(default = "bool_true")]
    pub enabled: bool,
    #[serde(default)]
    pub health: PriceServerHealthCheckConfig,
    #[serde(default)]
    pub server: PriceServerConfig,
    #[serde(default)]
    pub fees: FeeCalculatorConfig,
    #[serde(default)]
    pub price_cache: ExchangePriceCacheConfig,
}
impl Default for PriceServerWrapper {
    fn default() -> Self {
        Self {
            enabled: true,
            server: PriceServerConfig::default(),
            health: PriceServerHealthCheckConfig::default(),
            fees: FeeCalculatorConfig::default(),
            price_cache: ExchangePriceCacheConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceFeedConfigWrapper {
    #[serde(default = "bool_true")]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KolliderPriceFeedConfigWrapper {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default)]
    pub config: KolliderPriceFeedConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTradesConfigWrapper {
    #[serde(default = "bool_true")]
    pub enabled: bool,
    #[serde(default)]
    pub config: UserTradesConfig,
}
impl Default for UserTradesConfigWrapper {
    fn default() -> Self {
        Self {
            enabled: true,
            config: UserTradesConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HedgingConfigWrapper {
    #[serde(default = "bool_true")]
    pub enabled: bool,
    #[serde(default)]
    pub config: HedgingAppConfig,
}
impl Default for HedgingConfigWrapper {
    fn default() -> Self {
        Self {
            enabled: true,
            config: HedgingAppConfig::default(),
        }
    }
}

fn bool_true() -> bool {
    true
}
