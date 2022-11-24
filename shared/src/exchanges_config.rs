use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug};

pub type ExchangesConfig = HashMap<String, ExchangeConfigEntry>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExchangeConfigEntry {
    pub weight: Decimal,
    pub config: ExchangeType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum ExchangeType {
    #[serde(rename = "okex")]
    Okex(OkexConfig),
    #[serde(rename = "kollider")]
    Kollider(KolliderConfig),
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KolliderConfig {
    pub api_key: String,
    pub url: String,
}

#[cfg(test)]
mod test_super {
    use super::*;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;

    #[test]
    fn test_deserialize() {
        let str = r#"
                  okex: 
                    weight: 0.8
                    config:
                        type: okex
                        api_key: okex api
                  kollider: 
                    weight: 0.2
                    config:
                        type: kollider
                        api_key: kollider key
                        url: url
             "#;
        let ex: ExchangesConfig = serde_yaml::from_str(str).unwrap();
        let okex_item = ex.get("okex").unwrap();
        assert_eq!(dec!(0.8), okex_item.weight);
        dbg!(ex);
    }

    #[test]
    fn test_serialize() {
        let ok = ExchangeConfigEntry {
            weight: dec!(0.7),
            config: ExchangeType::Okex(OkexConfig {
                api_key: "okex api".to_string(),
                passphrase: "passphrase".to_string(),
                secret_key: "secret".to_string(),
                simulated: false,
            }),
        };

        let kollider = ExchangeConfigEntry {
            weight: dec!(0.3),
            config: ExchangeType::Kollider(KolliderConfig {
                api_key: "kollider key".to_string(),
                url: "url".to_string(),
            }),
        };

        let mut data = HashMap::new();
        data.insert("okex".to_string(), ok);
        data.insert("kollider".to_string(), kollider);

        let result = serde_yaml::to_string(&data).unwrap();
        println!("{:#?}", result);
    }
}
