mod error;
mod okex_response;

use chrono::{SecondsFormat, Utc};
use data_encoding::BASE64;
use ring::hmac;

use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client as ReqwestClient;

pub use error::*;
use okex_response::*;

#[derive(Debug, PartialEq)]
pub struct DepositAddress {
    pub value: String,
}

pub struct OkexClientConfig {
    pub api_key: String,
    pub passphrase: String,
    pub secret_key: String,
}

pub struct OkexClient {
    client: ReqwestClient,
    config: OkexClientConfig,
}

impl OkexClient {
    pub fn new(config: OkexClientConfig) -> Self {
        Self {
            client: ReqwestClient::new(),
            config,
        }
    }

    pub async fn get_funding_deposit_address(&self) -> Result<DepositAddress, OkexClientError> {
        let request_path = "/api/v5/asset/deposit-address?ccy=BTC";
        let base_url = "https://www.okx.com";
        let url = format!("{}{}", base_url, request_path);

        let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        let pre_hash_str = format!("{}GET{}", timestamp, request_path);
        let key = hmac::Key::new(hmac::HMAC_SHA256, self.config.secret_key.as_bytes());
        let signature = hmac::sign(&key, pre_hash_str.as_bytes());
        let sign_base64 = BASE64.encode(signature.as_ref());

        let mut headers = HeaderMap::new();
        headers.insert(
            "OK-ACCESS-KEY",
            HeaderValue::from_str(self.config.api_key.as_str())?,
        );
        headers.insert(
            "OK-ACCESS-SIGN",
            HeaderValue::from_str(sign_base64.as_str())?,
        );
        headers.insert(
            "OK-ACCESS-TIMESTAMP",
            HeaderValue::from_str(timestamp.as_str())?,
        );
        headers.insert(
            "OK-ACCESS-PASSPHRASE",
            HeaderValue::from_str(self.config.passphrase.as_str())?,
        );

        let response = self.client.get(url).headers(headers).send().await?;
        let response_text = response.text().await?;

        let response = match serde_json::from_str::<OkexResponse>(&response_text)? {
            OkexResponse::WithData(response) => response,
            OkexResponse::WithoutData(response) => {
                return Err(OkexClientError::from(response));
            }
        };

        if let Some(data) = response.data.first() {
            Ok(DepositAddress {
                value: data.addr.clone(),
            })
        } else {
            Err(OkexClientError::UnexpectedResponse {
                msg: response.msg,
                code: response.code,
            })
        }
    }
}
