mod config;

use config::*;
use reqwest::Client as ReqwestClient;
use shared::time::TimeStamp;

#[derive(Debug)]
struct AuthHeaders {
    access_key: ApiKey,
    access_sign: SecretKey,
    access_passphrase: PassPhrase,
    access_timestamp: TimeStamp,
}

impl From<ApiConfig> for AuthHeaders {
    fn from(config: ApiConfig) -> Self {
        let access_key = config.api_key;
        let access_passphrase = config.passphrase;
        let access_timestamp = TimeStamp::now();
        let access_timestamp = todo!();

        Self {
            access_key,
            access_sign,
            access_passphrase,
            access_timestamp,
        }
    }
}

#[derive(Debug)]
pub struct OkexClient {
    client: ReqwestClient,
    headers: AuthHeaders,
}

impl OkexClient {
    fn new(config: ApiConfig) -> Self {
        Self {
            client: ReqwestClient::new(),
            headers: AuthHeaders::from(config),
        }
    }
}
