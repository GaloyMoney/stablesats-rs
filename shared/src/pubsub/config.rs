use serde::Deserialize;

#[derive(Clone, Deserialize, Debug)]
pub struct PubSubConfig {
    pub host: Option<String>,
}

impl Default for PubSubConfig {
    fn default() -> Self {
        Self {
            host: Some("localhost".to_string()),
        }
    }
}
