use std::num::NonZeroU32;
use std::time::Duration;

use chrono::SecondsFormat;
use chrono::Utc;
use data_encoding::BASE64;
use reqwest::Client;

use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use reqwest::header::CONTENT_TYPE;
use reqwest::Response;
use reqwest::StatusCode;
use ring::hmac;
use serde_derive::Deserialize;
use serde_derive::Serialize;

use self::error::KolliderClientError;
pub use self::primitives::*;

mod error;
mod primitives;

use governor::{
    clock::DefaultClock, state::keyed::DefaultKeyedStateStore, Jitter, Quota, RateLimiter,
};

lazy_static::lazy_static! {
    static ref LIMITER: RateLimiter<&'static str, DefaultKeyedStateStore<&'static str>, DefaultClock>  = RateLimiter::keyed(Quota::per_second(NonZeroU32::new(1).unwrap()));
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct KolliderClientConfig {
    pub url: String,
    pub api_key: String,
    pub passphrase: String,
    pub secret: String,
}

#[derive(Clone)]
pub struct KolliderClient {
    client: Client,
    config: KolliderClientConfig,
}

impl KolliderClient {
    pub fn new(cfg: KolliderClientConfig) -> KolliderClient {
        KolliderClient {
            client: reqwest::Client::new(),
            config: cfg,
        }
    }

    pub async fn rate_limit_client(&self, key: &'static str) -> &Client {
        let jitter = Jitter::new(Duration::from_secs(1), Duration::from_secs(1));
        LIMITER.until_key_ready_with_jitter(&key, jitter).await;
        &self.client
    }

    fn create_headers(
        &self,
        timestamp: &str,
        signature: &str,
    ) -> Result<HeaderMap, KolliderClientError> {
        let mut header = HeaderMap::new();
        header.append(CONTENT_TYPE, HeaderValue::from_str("application/json")?);
        header.append("k-signature", HeaderValue::from_str(signature)?);

        header.append("k-timestamp", HeaderValue::from_str(timestamp)?);
        header.append(
            "k-passphrase",
            HeaderValue::from_str(&self.config.passphrase)?,
        );
        header.append("k-api-key", HeaderValue::from_str(&self.config.api_key)?);
        Ok(header)
    }

    fn generate_signature(secretb64: &str, pre_hash: &str) -> Result<String, KolliderClientError> {
        let res = BASE64.decode(secretb64.as_bytes())?;
        let key = hmac::Key::new(hmac::HMAC_SHA256, &res);
        let signature = hmac::sign(&key, pre_hash.as_bytes());
        let sig_encoded = BASE64.encode(signature.as_ref());
        Ok(sig_encoded)
    }

    fn create_get_headers(&self, path: &str) -> Result<HeaderMap, KolliderClientError> {
        let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        let pre_hash = format!("{}{}{}", timestamp, "GET", path);
        let sig = Self::generate_signature(&self.config.secret, &pre_hash)?;
        Self::create_headers(self, &timestamp, &sig)
    }

    fn create_post_headers(
        &self,
        path: &str,
        body: &str,
    ) -> Result<HeaderMap, KolliderClientError> {
        let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        let pre_hash = format!("{}{}{}{}", timestamp, "POST", path, body);
        let sig = Self::generate_signature(&self.config.secret, &pre_hash)?;
        Self::create_headers(self, &timestamp, &sig)
    }

    async fn extract_response_data<T: serde::de::DeserializeOwned>(
        response: Response,
    ) -> Result<T, KolliderClientError> {
        dbg!(&response.status());
        match response.status() {
            StatusCode::OK | StatusCode::CREATED => {
                let response_text = response.text().await?;
                dbg!(response_text.clone());
                match serde_json::from_str::<T>(&response_text) {
                    Ok(data) => Ok(data),
                    Err(err) => Err(KolliderClientError::UnexpectedResponse(err.to_string())),
                }
            }
            _ => {
                let response_text = response.text().await?;
                dbg!(response_text.clone());
                let data = serde_json::from_str::<KolliderErrorResponse>(&response_text)?;
                Err(KolliderClientError::UnexpectedResponse(format!(
                    "{} {}",
                    data.error, data.message
                )))
            }
        }
    }

    pub async fn get_user_balances(&self) -> Result<UserBalances, KolliderClientError> {
        let path = "/user/balances";
        let res = self
            .rate_limit_client(path)
            .await
            .get(format!("{}{}", &self.config.url, path))
            .headers(Self::create_get_headers(self, path)?)
            .send()
            .await?;
        Self::extract_response_data::<UserBalances>(res).await
    }

    pub async fn get_products(&self) -> Result<Products, KolliderClientError> {
        let path = "/market/products";
        let res = self
            .rate_limit_client(path)
            .await
            .get(format!("{}{}", &self.config.url, path))
            .send()
            .await?;

        Self::extract_response_data::<Products>(res).await
    }

    pub async fn make_deposit(&self, sats: i32) -> Result<PaymentRequest, KolliderClientError> {
        let path = "/wallet/deposit";

        let request_body = serde_json::json!({
            "type": "Ln",
            "amount": sats,
        })
        .to_string();

        let res = self
            .rate_limit_client(path)
            .await
            .post(format!("{}{}", self.config.url, path))
            .headers(Self::create_post_headers(self, path, &request_body)?)
            .body(request_body)
            .send()
            .await?;
        Self::extract_response_data::<PaymentRequest>(res).await
    }

    pub async fn make_withdrawal(
        &self,
        amount: i32,
        payment_request: &str,
    ) -> Result<String, KolliderClientError> {
        let path = "/wallet/withdrawal";

        let request_body = serde_json::json!({
            "type": "Ln",
            "payment_request": payment_request,
            "amount": amount,
        })
        .to_string();

        let res = self
            .rate_limit_client(path)
            .await
            .post(format!("{}{}", self.config.url, path))
            .headers(Self::create_post_headers(self, path, &request_body)?)
            .body(request_body)
            .send()
            .await?;
        Ok(res.text().await?)
    }

    pub async fn place_order(
        &self,
        order_side: KolliderOrderSide,
        amount_usd: i32,
        leverage_percent: i32,
    ) -> Result<PlaceOrderResult, KolliderClientError> {
        let path = "/orders";

        let request_body = serde_json::json!({
            "price": 0,
            "order_type": KolliderOrderType::Market.to_string(),
            "side": order_side.to_string(),
            "quantity": amount_usd,
            "symbol": KolliderInstrumentId::BtcUsdSwap.to_string(),
            "leverage": leverage_percent,
            "margin_type": KolliderMarginType::Cross.to_string(),
            "settlement_type": "Delayed"
        })
        .to_string();

        let res = self
            .rate_limit_client(path)
            .await
            .post(format!("{}{}", self.config.url, path))
            .headers(Self::create_post_headers(self, path, &request_body)?)
            .body(request_body)
            .send()
            .await?;

        Self::extract_response_data::<PlaceOrderResult>(res).await
    }

    pub async fn get_open_positions(&self) -> Result<OpenPositions, KolliderClientError> {
        let path = "/positions";
        let res = self
            .rate_limit_client(path)
            .await
            .get(format!("{}{}", self.config.url, path))
            .headers(Self::create_get_headers(self, path)?)
            .send()
            .await?;
        Self::extract_response_data::<OpenPositions>(res).await
    }

    pub async fn get_open_orders(&self) -> Result<OpenOrders, KolliderClientError> {
        let path = "/orders/open";
        let url = format!("{}{}", self.config.url, path);

        let res = self
            .rate_limit_client(path)
            .await
            .get(url)
            .headers(Self::create_get_headers(self, path)?)
            .send()
            .await?;
        Self::extract_response_data::<OpenOrders>(res).await
    }
}
