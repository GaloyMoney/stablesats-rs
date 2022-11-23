use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug};

pub type ExchangesConfig = HashMap<String, ExchangeConfigEntry>;

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExchangeConfigEntry {
    pub weight: Decimal,
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
            config: ExchangeType::OkEx(OkExConfig {
                api_key: "okex api".to_string(),
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
