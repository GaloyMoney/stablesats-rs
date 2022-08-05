#[derive(Debug)]
pub struct ApiConfig {
    pub api_key: Option<String>,
    pub secret_key: Option<String>,
    pub passphrase: Option<String>,
}
