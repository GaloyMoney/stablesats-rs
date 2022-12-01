use rust_decimal::Decimal;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ExchangeConfigAll {
    pub okex: Option<ExchangeConfig<OkexConfig>>,
    pub kollider: Option<ExchangeConfig<KolliderConfig>>,
}

impl ExchangeConfigAll {
    pub fn is_valid(&self) -> bool {
        self.okex.is_some() || self.kollider.is_some()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExchangeConfig<T: DeserializeOwned + Serialize> {
    pub weight: Decimal,
    #[serde(bound = "T: DeserializeOwned")]
    pub config: T,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct OkexConfig {
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub passphrase: String,
    #[serde(default)]
    pub secret_key: String,
    #[serde(default)]
    pub simulated: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct KolliderConfig {
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub url: String,
}

#[cfg(test)]
mod test_super {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_default() {
        let config = ExchangeConfigAll::default();
        assert!(config.okex.is_none());
        assert!(config.kollider.is_none());
    }

    #[test]
    fn test_deserialize() {
        let str = r#"
                  okex: 
                    weight: 0.8
                    config:
                        type: okex
                        api_key: okex api key
                  kollider: 
                    weight: 0.2
                    config:
                        type: kollider
                        api_key: kollider api key
                        url: url
             "#;
        let ex: ExchangeConfigAll = serde_yaml::from_str(str).expect("Couldn't deserialize yaml");

        let okex = ex.okex.expect("Okex-config not found");
        assert_eq!(dec!(0.8), okex.weight);
        assert_eq!("okex api key", okex.config.api_key);

        let kollider = ex.kollider.expect("Kollider-config not found");
        assert_eq!(dec!(0.2), kollider.weight);
        assert_eq!("kollider api key", kollider.config.api_key);
    }

    #[test]
    fn test_serialize() -> anyhow::Result<()> {
        let ok = ExchangeConfig {
            weight: dec!(0.8),
            config: OkexConfig {
                api_key: "okex api key".to_string(),
                passphrase: "okex passphrase".to_string(),
                secret_key: "okex secret key".to_string(),
                simulated: false,
            },
        };

        let kollider = ExchangeConfig {
            weight: dec!(0.8),
            config: KolliderConfig {
                api_key: "kollider api key".to_string(),
                url: "kollider url".to_string(),
            },
        };

        let data = ExchangeConfigAll {
            okex: Some(ok),
            kollider: Some(kollider),
        };
        let result = serde_yaml::to_string(&data)?;
        assert!(result.contains("okex passphrase"));
        Ok(())
    }

    #[test]
    fn test_deserialize_new() -> anyhow::Result<()> {
        let str = r#"
                  okex:
                    weight: 1
                    config:
                        api_key: okex api
                  kollider:
                    weight: 2
                    config:
                        api_key: kollider key
                        url: url
             "#;
        let ex: ExchangeConfigAll = serde_yaml::from_str(str)?;
        let okex_cfg = ex.okex.expect("Okex-config not found");
        assert_eq!(dec!(1), okex_cfg.weight);
        let kollider_cfg = ex.kollider.expect("Kollider-config not found");
        assert_eq!(dec!(2), kollider_cfg.weight);
        Ok(())
    }
}
