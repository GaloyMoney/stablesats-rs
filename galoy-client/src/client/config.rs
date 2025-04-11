use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GaloyClientConfig {
    #[serde(default)]
    pub api: String,
    #[serde(default)]
    pub api_key: String,
}
