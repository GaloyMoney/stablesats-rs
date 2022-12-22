mod bitfinex_response;
mod error;
mod primitives;

use chrono::{SecondsFormat, Utc};
use data_encoding::HEXLOWER;
use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_TYPE},
    Client as ReqwestClient, Response,
};
use ring::hmac;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use std::{collections::HashMap, time::Duration};

use bitfinex_response::*;
pub use error::*;
pub use primitives::*;

use governor::{
    clock::DefaultClock, state::keyed::DefaultKeyedStateStore, Jitter, Quota, RateLimiter,
};
use std::num::NonZeroU32;

lazy_static::lazy_static! {
    static ref LIMITER: RateLimiter<&'static str, DefaultKeyedStateStore<&'static str>, DefaultClock>  = RateLimiter::keyed(Quota::per_second(NonZeroU32::new(1).unwrap()));
}

const REST_API_V2_URL: &str = "https://api.bitfinex.com/v2";
const REST_API_SIGNATURE_PATH: &str = "/api/v2/auth/r/";

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct BitfinexClientConfig {
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub secret_key: String,
    #[serde(default)]
    pub simulated: bool,
}

#[derive(Clone)]
pub struct BitfinexClient {
    client: ReqwestClient,
    config: BitfinexClientConfig,
}

impl BitfinexClient {
    pub async fn new(config: BitfinexClientConfig) -> Result<Self, BitfinexClientError> {
        let client = Self {
            client: ReqwestClient::builder().use_rustls_tls().build()?,
            config,
        };
        // let path = "/api/v5/account/config";
        // let config_url = Self::url_for_path(path);
        // let headers = client.get_request_headers(path)?;

        // let response = client
        //     .rate_limit_client(path)
        //     .await
        //     .get(config_url)
        //     .headers(headers)
        //     .send()
        //     .await?;
        // let config_data =
        //     Self::extract_response_data::<BitfinexAccountConfigurationData>(response).await?;

        // if &config_data.pos_mode != "net_mode" {
        //     return Err(BitfinexClientError::MisconfiguredAccount(format!(
        //         "Expected `net_mode`, got `{}`",
        //         config_data.pos_mode
        //     )));
        // }

        // if &config_data.acct_lv != "2" {
        //     return Err(BitfinexClientError::MisconfiguredAccount(format!(
        //         "Expected `acct_lv: 2`, got `{}`",
        //         config_data.acct_lv
        //     )));
        // }
        Ok(client)
    }

    pub async fn rate_limit_client(&self, key: &'static str) -> &ReqwestClient {
        let jitter = Jitter::new(Duration::from_secs(1), Duration::from_secs(1));
        LIMITER.until_key_ready_with_jitter(&key, jitter).await;
        &self.client
    }

    #[instrument(skip(self), err)]
    pub async fn get_last_price_in_usd_cents(&self) -> Result<LastPrice, BitfinexClientError> {
        let endpoint = "/ticker";
        let params = format!("/{}", BitfinexInstrumentId::BtcUsdSwap);

        let response = self
            .rate_limit_client(endpoint)
            .await
            .get(Self::url_for_path(endpoint, params.as_str()))
            .send()
            .await?;

        if let Some(LastPriceData { last_price, .. }) =
            Self::extract_response_data::<LastPriceData>(response).await?
        {
            Ok(LastPrice {
                usd_cents: last_price * Decimal::ONE_HUNDRED,
            })
        } else {
            Err(BitfinexClientError::NoLastPriceAvailable)
        }
    }

    #[instrument(skip(self), err)]
    pub async fn funding_info(&self) -> Result<FundingInfo, BitfinexClientError> {
        let body: HashMap<String, String> = HashMap::new();
        // body.insert(
        //     "instId".to_string(),
        //     BitfinexInstrumentId::BtcUsdSwap.to_string(),
        // );
        let request_body = serde_json::to_string(&body)?;

        let endpoint = "/info/user";
        let params = format!("/{}", BitfinexInstrumentId::UsdSpot);
        let headers = self.post_request_headers(endpoint, &request_body)?;

        let response = self
            .rate_limit_client(endpoint)
            .await
            .post(Self::url_for_auth_path(endpoint, params.as_str()))
            .headers(headers)
            .body(request_body)
            .send()
            .await?;

        if let Some(FundingInfoData {
            key,
            symbol,
            funding,
        }) = Self::extract_response_data::<FundingInfoData>(response).await?
        {
            Ok(FundingInfo {
                key,
                symbol,
                yield_loan: funding.yield_loan,
                yield_lend: funding.yield_lend,
                duration_loan: funding.duration_loan,
                duration_lend: funding.duration_lend,
            })
        } else {
            Err(BitfinexClientError::NoLastPriceAvailable)
        }
    }

    async fn extract_response_data<T: serde::de::DeserializeOwned>(
        response: Response,
    ) -> Result<Option<T>, BitfinexClientError> {
        let response_text = response.text().await?;
        let BitfinexResponse {
            message,
            code,
            data,
        } = serde_json::from_str::<BitfinexResponse<T>>(&response_text)?;
        if let Some(data) = data {
            if let Some(first) = data.into_iter().next() {
                return Ok(Some(first));
            } else {
                return Ok(None);
            }
        }
        let msg = message.unwrap_or_default();
        let code = code.unwrap_or_default();
        Err(BitfinexClientError::from((msg, code)))
    }

    // async fn extract_optional_response_data<T: serde::de::DeserializeOwned>(
    //     response: Response,
    // ) -> Result<Option<T>, BitfinexClientError> {
    //     let response_text = response.text().await?;
    //     let BitfinexResponse { code, msg, data } =
    //         serde_json::from_str::<BitfinexResponse<T>>(&response_text)?;
    //     if let Some(data) = data {
    //         if let Some(first) = data.into_iter().next() {
    //             return Ok(Some(first));
    //         } else {
    //             return Ok(None);
    //         }
    //     }
    //     Err(BitfinexClientError::from((msg, code)))
    // }

    fn url_for_path(endpoint: &str, params: &str) -> String {
        format!("{}{}{}", REST_API_V2_URL, endpoint, params)
    }

    fn url_for_auth_path(endpoint: &str, params: &str) -> String {
        format!("{}/auth/r/{}{}", REST_API_V2_URL, endpoint, params)
    }

    // pub async fn post_authenticated(
    //     &self,
    //     request_path: &str,
    //     request_body: &str,
    // ) -> Result<Response, reqwest::Error> {
    //     let headers = self.post_request_headers(request_path, &request_body)?;
    //     self.rate_limit_client(request_path)
    //         .await
    //         .post(Self::url_for_auth_path(request_path))
    //         .headers(headers)
    //         .body(request_body)
    //         .query(&[])
    //         .send()
    //         .await
    // }

    fn sign_request(&self, pre_hash: String) -> String {
        let key = hmac::Key::new(hmac::HMAC_SHA384, self.config.secret_key.as_bytes());
        let signature = hmac::sign(&key, pre_hash.as_bytes());
        let sig = HEXLOWER.encode(signature.as_ref());
        let expected = hex::encode(signature.as_ref());
        assert_eq!(sig, expected);
        // todo!()
        sig
    }

    fn post_request_headers(
        &self,
        request: &str,
        body: &str,
    ) -> Result<HeaderMap, BitfinexClientError> {
        let nonce = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        let pre_hash: String = format!("{}{}{}{}", REST_API_SIGNATURE_PATH, request, nonce, body);

        let signature = self.sign_request(pre_hash);

        let mut headers = HeaderMap::new();
        headers.insert("bfx-nonce", HeaderValue::from_str(nonce.as_str())?);
        headers.insert(
            "bfx-apikey",
            HeaderValue::from_str(self.config.api_key.as_str())?,
        );
        headers.insert("bfx-signature", HeaderValue::from_str(signature.as_str())?);
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        Ok(headers)
    }
}
