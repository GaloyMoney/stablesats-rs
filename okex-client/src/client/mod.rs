mod error;
mod okex_response;

use std::collections::HashMap;

use chrono::{SecondsFormat, Utc};
use data_encoding::BASE64;
use ring::hmac;

use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use reqwest::Client as ReqwestClient;

pub use error::*;
use okex_response::*;

const OKEX_API_URL: &str = "https://www.okex.com";

#[derive(Debug, PartialEq)]
pub struct DepositAddress {
    pub value: String,
}

#[derive(Debug)]
pub struct TransferId {
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
        let url = format!("{}{}", OKEX_API_URL, request_path);

        let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        let pre_hash = format!("{}GET{}", timestamp, request_path);

        let sign_base64 = self.sign_okex_request(pre_hash);

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
            match data {
                OkexResponseData::DepositAddress(addr_data) => Ok(DepositAddress {
                    value: addr_data.addr.clone(),
                }),
                _ => Err(OkexClientError::UnexpectedResponse {
                    msg: response.msg,
                    code: response.code,
                }),
            }
        } else {
            Err(OkexClientError::UnexpectedResponse {
                msg: response.msg,
                code: response.code,
            })
        }
    }

    pub async fn transfer(&self, amt: f64) -> Result<TransferId, OkexClientError> {
        let request_path = "/api/v5/asset/transfer";
        let url = format!("{}{}", OKEX_API_URL, request_path);

        let mut body: HashMap<String, String> = HashMap::new();
        body.insert("ccy".to_string(), "BTC".to_string());
        body.insert("amt".to_string(), amt.to_string());
        body.insert("from".to_string(), "6".to_string());
        body.insert("to".to_string(), "18".to_string());
        let request_body = serde_json::to_string(&body)?;

        let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        let pre_hash = format!("{}POST{}{}", timestamp, request_path, request_body);
        let sign_base64 = self.sign_okex_request(pre_hash);

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_str("application/json")?);
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

        let response = self
            .client
            .post(url)
            .headers(headers)
            .body(request_body)
            .send()
            .await?;
        let response_text = response.text().await?;

        let response = match serde_json::from_str::<OkexResponse>(&response_text)? {
            OkexResponse::WithData(response) => response,
            OkexResponse::WithoutData(response) => {
                return Err(OkexClientError::from(response));
            }
        };

        if let Some(data) = response.data.first() {
            match data {
                OkexResponseData::Transfer(trans_data) => Ok(TransferId {
                    value: trans_data.trans_id.clone(),
                }),
                _ => Err(OkexClientError::UnexpectedResponse {
                    msg: response.msg,
                    code: response.code,
                }),
            }
        } else {
            Err(OkexClientError::UnexpectedResponse {
                msg: response.msg,
                code: response.code,
            })
        }
    }

    fn sign_okex_request(&self, pre_hash: String) -> String {
        let key = hmac::Key::new(hmac::HMAC_SHA256, self.config.secret_key.as_bytes());
        let signature = hmac::sign(&key, pre_hash.as_bytes());
        BASE64.encode(signature.as_ref())
    }
}
