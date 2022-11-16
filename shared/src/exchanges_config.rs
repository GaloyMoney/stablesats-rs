use std::fmt::Debug;

use serde::{Deserialize, Serialize};

pub type ExchangesConfig = Vec<ExchangeConfigEntry>;

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExchangeConfigEntry {
    pub weight: i32,
    pub config: ExchangeType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum ExchangeType {
    #[serde(rename = "okex")]
    OkEx(OkExConfig),
    #[serde(rename = "kollider")]
    Kollider(KolliderConfig),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OkExConfig {
    pub api_key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KolliderConfig {
    pub api_key: String,
    pub url: String,
}

#[cfg(test)]
mod test_super {
    use super::*;

    #[test]
    fn test_deserialize() {
        let str = r#"
                  - weight: 50
                    config:
                        type: okex
                        api_key: okex api
                  - weight: 50
                    config:
                        type: kollider
                        api_key: kollider key
                        url: url
             "#;
        let ex: ExchangesConfig = serde_yaml::from_str(str).unwrap();
        println!("ex {:#?}", ex);
    }

    #[test]
    fn test_serialize() {
        let ok = ExchangeConfigEntry {
            weight: 50,
            config: ExchangeType::OkEx(OkExConfig {
                api_key: "okex api".to_string(),
            }),
        };

        let kollider = ExchangeConfigEntry {
            weight: 50,
            config: ExchangeType::Kollider(KolliderConfig {
                api_key: "kollider key".to_string(),
                url: "url".to_string(),
            }),
        };

        let lst = vec![ok, kollider];
        let result = serde_yaml::to_string(&lst).unwrap();
        println!("{:#?}", result);
    }
}
