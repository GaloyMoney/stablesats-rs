use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PriceServerAppConfig {
    #[serde(default = "default_use_okex")]
    pub use_okex: bool,
}

fn default_use_okex() -> bool {
    true
}

impl Default for PriceServerAppConfig {
    fn default() -> Self {
        Self {
            use_okex: default_use_okex(),
        }
    }
}
