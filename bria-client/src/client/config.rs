use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct BriaClientConfig {
    #[serde(default = "default_url")]
    pub url: String,
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub wallet_name: String,
    #[serde(default)]
    pub payout_queue_name: String,
    #[serde(default)]
    pub external_id: String,
}

fn default_url() -> String {
    "http://localhost:2742".to_string()
}
