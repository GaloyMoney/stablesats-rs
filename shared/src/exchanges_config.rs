use rust_decimal::Decimal;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ExchangeConfigs {
    pub okex: Option<ExchangeConfig<OkexConfig>>,
    pub bitfinex: Option<ExchangeConfig<BitfinexConfig>>,
    pub deribit: Option<ExchangeConfig<DeribitConfig>>,
    pub kollider: Option<ExchangeConfig<KolliderConfig>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExchangeConfig<T: DeserializeOwned + Serialize + Default> {
    pub weight: Decimal,
    #[serde(bound = "T: DeserializeOwned")]
    #[serde(default)]
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
pub struct BitfinexConfig {
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub secret_key: String,
    #[serde(default)]
    pub simulated: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DeribitConfig {
    #[serde(default)]
    pub api_key: String,
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
        let config = ExchangeConfigs::default();
        assert!(config.okex.is_none());
        assert!(config.kollider.is_none());
    }

    #[test]
    fn test_deserialize() {
        let str = r#"
                  okex: 
                    weight: 0.8
                    config:
                        api_key: okex api key
                  bitfinex: 
                    weight: 0.2
                    config:
                        api_key: bitfinex api key
                  deribit: 
                    weight: 0.8
                    config:
                        api_key: deribit client id
                  kollider: 
                    weight: 0.2
                    config:
                        api_key: kollider api key
                        url: url
             "#;
        let ex: ExchangeConfigs = serde_yaml::from_str(str).expect("Couldn't deserialize yaml");

        let okex = ex.okex.expect("Okex-config not found");
        assert_eq!(dec!(0.8), okex.weight);
        assert_eq!("okex api key", okex.config.api_key);

        let bitfinex = ex.bitfinex.expect("Bitfinex-config not found");
        assert_eq!(dec!(0.2), bitfinex.weight);
        assert_eq!("bitfinex api key", bitfinex.config.api_key);

        let deribit = ex.deribit.expect("Deribit-config not found");
        assert_eq!(dec!(0.8), deribit.weight);
        assert_eq!("deribit api key", deribit.config.api_key);

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

        let bit = ExchangeConfig {
            weight: dec!(0.8),
            config: BitfinexConfig {
                api_key: "bitfinex api key".to_string(),
                secret_key: "bitfinex secret key".to_string(),
                simulated: false,
            },
        };

        let deribit = ExchangeConfig {
            weight: dec!(0.8),
            config: DeribitConfig {
                api_key: "deribit client id".to_string(),
                secret_key: "deribit secret key".to_string(),
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

        let data = ExchangeConfigs {
            okex: Some(ok),
            bitfinex: Some(bit),
            deribit: Some(deribit),
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
                  bitfinex:
                    weight: 3
                    config:
                        api_key: bitfinex api
                  deribit:
                    weight: 4
                    config:
                        api_key: deribit api
                  kollider:
                    weight: 2
                    config:
                        api_key: kollider key
                        url: url
             "#;
        let ex: ExchangeConfigs = serde_yaml::from_str(str)?;
        let okex_cfg = ex.okex.expect("Okex-config not found");
        assert_eq!(dec!(1), okex_cfg.weight);
        let bitfinex_cfg = ex.bitfinex.expect("Bitfinex-config not found");
        assert_eq!(dec!(3), bitfinex_cfg.weight);
        let deribit_cfg = ex.deribit.expect("Deribit-config not found");
        assert_eq!(dec!(4), deribit_cfg.weight);
        let kollider_cfg = ex.kollider.expect("Kollider-config not found");
        assert_eq!(dec!(2), kollider_cfg.weight);
        Ok(())
    }
}
