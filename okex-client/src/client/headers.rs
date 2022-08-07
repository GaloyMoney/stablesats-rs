use chrono::{DateTime, SecondsFormat, Utc};
use reqwest::header::{HeaderMap, HeaderValue};

use crate::ApiConfig;

use super::error::OkexClientError;

#[derive(Debug)]
pub struct AuthHeaders {
    pub headers: HeaderMap,
}

impl AuthHeaders {
    pub fn create(
        signature: String,
        client_config: ApiConfig,
        request_timestamp: DateTime<Utc>,
    ) -> Result<Self, OkexClientError> {
        let mut headers = HeaderMap::new();
        let ok_access_key = client_config.api_key.to_string();
        let ok_access_sign = signature;
        let ok_access_passphrase = client_config.pass_phrase;
        let ok_access_timestamp = request_timestamp.to_rfc3339_opts(SecondsFormat::Millis, true);

        let header_key_value = HeaderValue::from_str(ok_access_key.as_str())?;
        let header_sign_value = HeaderValue::from_str(ok_access_sign.as_str())?;
        let header_passphrase_value = HeaderValue::from_str(ok_access_passphrase.as_str())?;
        let header_timestamp_value = HeaderValue::from_str(ok_access_timestamp.as_str())?;

        headers.insert("OK-ACCESS-KEY", header_key_value);
        headers.insert("OK-ACCESS-SIGN", header_sign_value);
        headers.insert("OK-ACCESS-PASSPHRASE", header_passphrase_value);
        headers.insert("OK-ACCESS-TIMESTAMP", header_timestamp_value);

        Ok(Self { headers })
    }
}
