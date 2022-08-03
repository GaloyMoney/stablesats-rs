use serde::Deserialize;

#[derive(Clone, Deserialize, Debug)]
pub struct PubSubConfig {
    pub host: Option<String>,
}
