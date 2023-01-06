mod bitfinex_response;
mod error;
mod primitives;

use chrono::Utc;
use data_encoding::HEXLOWER;
use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_TYPE},
    Client as ReqwestClient, Response, StatusCode,
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
const REST_API_R_SIGNATURE_PATH: &str = "/api/v2/auth/r";
const REST_API_W_SIGNATURE_PATH: &str = "/api/v2/auth/w";

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

        Ok(client)
    }

    pub async fn rate_limit_client(&self, key: &'static str) -> &ReqwestClient {
        let jitter = Jitter::new(Duration::from_secs(1), Duration::from_secs(1));
        LIMITER.until_key_ready_with_jitter(&key, jitter).await;
        &self.client
    }

    pub async fn get_last_price_in_usd_cents(&self) -> Result<LastPrice, BitfinexClientError> {
        let endpoint = "/ticker";
        let params = format!("/{}", Instrument::BtcUsdSwap);

        let response = self
            .rate_limit_client(endpoint)
            .await
            .get(Self::url_for_path(endpoint, params.as_str()))
            .send()
            .await?;

        let data = Self::extract_response_data::<LastPriceData>(response).await?;
        Ok(LastPrice {
            usd_cents: data.last_price * Decimal::ONE_HUNDRED,
        })
    }

    #[instrument(skip(self), err)]
    pub async fn funding_info(&self) -> Result<FundingInfo, BitfinexClientError> {
        let body: HashMap<String, String> = HashMap::new();
        let request_body = serde_json::to_string(&body)?;

        let endpoint = "/info/funding";
        let params = format!("/{}", Instrument::UsdSpot);
        let headers = self.post_r_request_headers(endpoint, params.as_str(), &request_body)?;

        let response = self
            .rate_limit_client(endpoint)
            .await
            .post(Self::url_for_auth_r_path(endpoint, params.as_str()))
            .headers(headers)
            .body(request_body)
            .send()
            .await?;

        let FundingInfoData {
            key,
            symbol,
            funding,
        } = Self::extract_response_data::<FundingInfoData>(response).await?;
        Ok(FundingInfo {
            key,
            symbol,
            yield_loan: funding.yield_loan,
            yield_lend: funding.yield_lend,
            duration_loan: funding.duration_loan,
            duration_lend: funding.duration_lend,
        })
    }

    #[instrument(skip(self), err)]
    pub async fn get_orders(&self) -> Result<Vec<OrderDetails>, BitfinexClientError> {
        let body: HashMap<String, String> = HashMap::new();
        let request_body = serde_json::to_string(&body)?;

        let endpoint = "/orders";
        let params = format!("/{}/hist", Instrument::BtcUsdSwap);
        let headers = self.post_r_request_headers(endpoint, params.as_str(), &request_body)?;

        let response = self
            .rate_limit_client(endpoint)
            .await
            .post(Self::url_for_auth_r_path(endpoint, params.as_str()))
            .headers(headers)
            .body(request_body)
            .send()
            .await?;

        let mut orders = Self::extract_response_data::<Vec<OrderDetails>>(response).await?;
        for mut details in &mut orders {
            if let Some(status) = details.order_status.clone() {
                if status == "EXECUTED" || status == "CANCELED" {
                    details.complete = true;
                }
            }
        }
        Ok(orders)
    }

    #[instrument(skip(self), err)]
    pub async fn get_wallets(&self) -> Result<Vec<WalletDetails>, BitfinexClientError> {
        let body: HashMap<String, String> = HashMap::new();
        let request_body = serde_json::to_string(&body)?;

        let endpoint = "/wallets";
        let params = "";
        let headers = self.post_r_request_headers(endpoint, params, &request_body)?;

        let response = self
            .rate_limit_client(endpoint)
            .await
            .post(Self::url_for_auth_r_path(endpoint, params))
            .headers(headers)
            .body(request_body)
            .send()
            .await?;

        let wallets = Self::extract_response_data::<Vec<WalletDetails>>(response).await?;
        Ok(wallets)
    }

    #[instrument(skip(self), err)]
    pub async fn get_positions(&self) -> Result<Vec<PositionDetails>, BitfinexClientError> {
        let body: HashMap<String, String> = HashMap::new();
        let request_body = serde_json::to_string(&body)?;

        let endpoint = "/positions";
        let params = "";
        let headers = self.post_r_request_headers(endpoint, params, &request_body)?;

        let response = self
            .rate_limit_client(endpoint)
            .await
            .post(Self::url_for_auth_r_path(endpoint, params))
            .headers(headers)
            .body(request_body)
            .send()
            .await?;

        let positions = Self::extract_response_data::<Vec<PositionDetails>>(response).await?;
        Ok(positions)
    }

    #[instrument(skip(self), err)]
    pub async fn get_btc_on_chain_deposit_address(
        &self,
    ) -> Result<DepositAddress, BitfinexClientError> {
        let mut body: HashMap<String, String> = HashMap::new();
        body.insert("wallet".to_string(), Wallet::FUNDING.to_string());
        body.insert("method".to_string(), AddressMethod::BITCOIN.to_string());
        let request_body = serde_json::to_string(&body)?;

        let endpoint = "/deposit/address";
        let params = "";
        let headers = self.post_w_request_headers(endpoint, params, &request_body)?;

        let response = self
            .rate_limit_client(endpoint)
            .await
            .post(Self::url_for_auth_w_path(endpoint, params))
            .headers(headers)
            .body(request_body)
            .send()
            .await?;

        let details = Self::extract_response_data::<DepositAddressDetails>(response).await?;
        Ok(details.address)
    }

    #[instrument(skip(self), err)]
    pub async fn get_ln_deposit_address(&self) -> Result<DepositAddress, BitfinexClientError> {
        let mut body: HashMap<String, String> = HashMap::new();
        body.insert("wallet".to_string(), Wallet::EXCHANGE.to_string());
        body.insert("method".to_string(), AddressMethod::LNX.to_string());
        let request_body = serde_json::to_string(&body)?;

        let endpoint = "/deposit/address";
        let params = "";
        let headers = self.post_w_request_headers(endpoint, params, &request_body)?;

        let response = self
            .rate_limit_client(endpoint)
            .await
            .post(Self::url_for_auth_w_path(endpoint, params))
            .headers(headers)
            .body(request_body)
            .send()
            .await?;

        let details = Self::extract_response_data::<DepositAddressDetails>(response).await?;
        Ok(details.address)
    }

    #[instrument(skip(self), err)]
    pub async fn get_ln_invoice(
        &self,
        client_id: ClientId,
        amount: Decimal,
    ) -> Result<InvoiceDetails, BitfinexClientError> {
        let mut body: HashMap<String, String> = HashMap::new();
        body.insert("wallet".to_string(), Wallet::EXCHANGE.to_string());
        body.insert("currency".to_string(), Currency::LNX.to_string());
        body.insert("amount".to_string(), amount.to_string());
        let request_body = serde_json::to_string(&body)?;

        let endpoint = "/deposit/invoice";
        let params = "";
        let headers = self.post_w_request_headers(endpoint, params, &request_body)?;

        let response = self
            .rate_limit_client(endpoint)
            .await
            .post(Self::url_for_auth_w_path(endpoint, params))
            .headers(headers)
            .body(request_body)
            .send()
            .await?;

        let invoice = Self::extract_response_data::<InvoiceDetails>(response).await?;
        Ok(invoice)
    }

    #[instrument(skip(self), err)]
    pub async fn transfer_funding_to_trading(
        &self,
        client_id: ClientId,
        amount: Decimal,
    ) -> Result<MessageId, BitfinexClientError> {
        let mut body: HashMap<String, String> = HashMap::new();
        body.insert("from".to_string(), Wallet::FUNDING.to_string());
        body.insert("to".to_string(), Wallet::TRADING.to_string());
        body.insert("currency".to_string(), Currency::UST.to_string());
        body.insert("currency_to".to_string(), Currency::USTF0.to_string());
        body.insert("amount".to_string(), amount.to_string());
        let request_body = serde_json::to_string(&body)?;

        let endpoint = "/transfer";
        let params = "";
        let headers = self.post_w_request_headers(endpoint, params, &request_body)?;

        let response = self
            .rate_limit_client(endpoint)
            .await
            .post(Self::url_for_auth_w_path(endpoint, params))
            .headers(headers)
            .body(request_body)
            .send()
            .await?;

        let transfer = Self::extract_response_data::<TransferDetails>(response).await?;
        Ok(transfer.message_id)
    }

    #[instrument(skip(self), err)]
    pub async fn transfer_trading_to_funding(
        &self,
        client_id: ClientId,
        amount: Decimal,
    ) -> Result<MessageId, BitfinexClientError> {
        let mut body: HashMap<String, String> = HashMap::new();
        body.insert("from".to_string(), Wallet::TRADING.to_string());
        body.insert("to".to_string(), Wallet::FUNDING.to_string());
        body.insert("currency".to_string(), Currency::USTF0.to_string());
        body.insert("currency_to".to_string(), Currency::UST.to_string());
        body.insert("amount".to_string(), amount.to_string());
        let request_body = serde_json::to_string(&body)?;

        let endpoint = "/transfer";
        let params = "";
        let headers = self.post_w_request_headers(endpoint, params, &request_body)?;

        let response = self
            .rate_limit_client(endpoint)
            .await
            .post(Self::url_for_auth_w_path(endpoint, params))
            .headers(headers)
            .body(request_body)
            .send()
            .await?;

        let transfer = Self::extract_response_data::<TransferDetails>(response).await?;
        Ok(transfer.message_id)
    }

    #[instrument(skip(self), err)]
    pub async fn withdraw_btc_onchain(
        &self,
        client_id: ClientId,
        amount: Decimal,
        fee: Decimal,
        address: String,
    ) -> Result<MessageId, BitfinexClientError> {
        let mut body: HashMap<String, String> = HashMap::new();
        body.insert("wallet".to_string(), Wallet::FUNDING.to_string());
        body.insert("method".to_string(), AddressMethod::BITCOIN.to_string());
        body.insert("amount".to_string(), amount.to_string());
        body.insert("address".to_string(), address.to_string());
        body.insert("payment_id".to_string(), client_id.0.to_string());
        body.insert("fee_deduct".to_string(), 0.to_string());
        let request_body = serde_json::to_string(&body)?;

        let endpoint = "/withdraw";
        let params = "";
        let headers = self.post_w_request_headers(endpoint, params, &request_body)?;

        let response = self
            .rate_limit_client(endpoint)
            .await
            .post(Self::url_for_auth_w_path(endpoint, params))
            .headers(headers)
            .body(request_body)
            .send()
            .await?;

        let transfer = Self::extract_response_data::<WithdrawDetails>(response).await?;
        Ok(transfer.message_id)
    }

    #[instrument(skip(self), err)]
    pub async fn withdraw_btc_on_ln(
        &self,
        client_id: ClientId,
        invoice: String,
    ) -> Result<MessageId, BitfinexClientError> {
        let mut body: HashMap<String, String> = HashMap::new();
        body.insert("wallet".to_string(), Wallet::FUNDING.to_string());
        body.insert("method".to_string(), AddressMethod::LNX.to_string());
        body.insert("invoice".to_string(), invoice.to_string());
        body.insert("payment_id".to_string(), client_id.0.to_string());
        body.insert("fee_deduct".to_string(), 0.to_string());
        let request_body = serde_json::to_string(&body)?;

        let endpoint = "/withdraw";
        let params = "";
        let headers = self.post_w_request_headers(endpoint, params, &request_body)?;

        let response = self
            .rate_limit_client(endpoint)
            .await
            .post(Self::url_for_auth_w_path(endpoint, params))
            .headers(headers)
            .body(request_body)
            .send()
            .await?;

        let transfer = Self::extract_response_data::<WithdrawDetails>(response).await?;
        Ok(transfer.message_id)
    }

    #[instrument(skip(self), err)]
    pub async fn get_ln_transactions(
        &self,
        client_id: ClientId,
    ) -> Result<Vec<TransactionDetails>, BitfinexClientError> {
        let body: HashMap<String, String> = HashMap::new();
        let request_body = serde_json::to_string(&body)?;

        let endpoint = "/movements";
        let params = format!("/{}/hist", Currency::LNX);
        let headers = self.post_r_request_headers(endpoint, params.as_str(), &request_body)?;

        let response = self
            .rate_limit_client(endpoint)
            .await
            .post(Self::url_for_auth_r_path(endpoint, params.as_str()))
            .headers(headers)
            .body(request_body)
            .send()
            .await?;

        let transactions = Self::extract_response_data::<Vec<TransactionDetails>>(response).await?;
        Ok(transactions)
    }

    #[instrument(skip(self), err)]
    pub async fn get_btc_on_chain_transactions(
        &self,
        client_id: ClientId,
    ) -> Result<Vec<TransactionDetails>, BitfinexClientError> {
        let body: HashMap<String, String> = HashMap::new();
        let request_body = serde_json::to_string(&body)?;

        let endpoint = "/movements";
        let params = format!("/{}/hist", Currency::BTC);
        let headers = self.post_r_request_headers(endpoint, params.as_str(), &request_body)?;

        let response = self
            .rate_limit_client(endpoint)
            .await
            .post(Self::url_for_auth_r_path(endpoint, params.as_str()))
            .headers(headers)
            .body(request_body)
            .send()
            .await?;

        let transactions = Self::extract_response_data::<Vec<TransactionDetails>>(response).await?;
        Ok(transactions)
    }

    #[instrument(skip(self), err)]
    pub async fn submit_order(
        &self,
        client_id: ClientId,
        amount: Decimal,
        leverage: Decimal,
    ) -> Result<SubmittedOrderDetails, BitfinexClientError> {
        let mut body: HashMap<String, String> = HashMap::new();
        body.insert("cid".to_string(), client_id.0.to_string());
        body.insert("type".to_string(), OrderType::MARKET.to_string());
        body.insert("symbol".to_string(), Instrument::BtcUsdSwap.to_string());
        body.insert("amount".to_string(), amount.to_string());
        body.insert("lev".to_string(), leverage.to_string());
        let request_body = serde_json::to_string(&body)?;

        let endpoint = "/order/submit";
        let params = "";
        let headers = self.post_w_request_headers(endpoint, params, &request_body)?;

        let response = self
            .rate_limit_client(endpoint)
            .await
            .post(Self::url_for_auth_w_path(endpoint, params))
            .headers(headers)
            .body(request_body)
            .send()
            .await?;

        let order = Self::extract_response_data::<SubmittedOrderDetails>(response).await?;
        Ok(order)
    }

    #[instrument(skip(self), err)]
    pub async fn get_api_key_permissions(&self) -> Result<Vec<ApiKeyDetails>, BitfinexClientError> {
        let body: HashMap<String, String> = HashMap::new();
        let request_body = serde_json::to_string(&body)?;

        let endpoint = "/permissions";
        let params = "";
        let headers = self.post_r_request_headers(endpoint, params, &request_body)?;

        let response = self
            .rate_limit_client(endpoint)
            .await
            .post(Self::url_for_auth_r_path(endpoint, params))
            .headers(headers)
            .body(request_body)
            .send()
            .await?;

        let transactions = Self::extract_response_data::<Vec<ApiKeyDetails>>(response).await?;
        Ok(transactions)
    }

    async fn extract_response_data<T: serde::de::DeserializeOwned>(
        response: Response,
    ) -> Result<T, BitfinexClientError> {
        match response.status() {
            StatusCode::OK => {
                let response_text = response.text().await?;
                dbg!(response_text.clone());
                match serde_json::from_str::<T>(&response_text) {
                    Ok(data) => Ok(data),
                    Err(..) => Err(BitfinexClientError::UnexpectedResponse {
                        msg: "".to_string(),
                        code: 0,
                    }),
                }
            }
            _ => {
                let response_text = response.text().await?;
                dbg!(response_text.clone());
                let data = serde_json::from_str::<BitfinexErrorResponse>(&response_text)?;
                Err(BitfinexClientError::from((data.message, data.code)))
            }
        }
    }

    fn url_for_path(endpoint: &str, params: &str) -> String {
        format!("{}{}{}", REST_API_V2_URL, endpoint, params)
    }

    fn url_for_auth_r_path(endpoint: &str, params: &str) -> String {
        format!("{}/auth/r{}{}", REST_API_V2_URL, endpoint, params)
    }

    fn url_for_auth_w_path(endpoint: &str, params: &str) -> String {
        format!("{}/auth/w{}{}", REST_API_V2_URL, endpoint, params)
    }

    fn sign_request(&self, pre_hash: String) -> String {
        let key = hmac::Key::new(hmac::HMAC_SHA384, self.config.secret_key.as_bytes());
        let signature = hmac::sign(&key, pre_hash.as_bytes());
        HEXLOWER.encode(signature.as_ref())
    }

    fn post_request_headers(
        &self,
        sig_path: &str,
        request: &str,
        params: &str,
        body: &str,
    ) -> Result<HeaderMap, BitfinexClientError> {
        let nonce = Utc::now().timestamp_millis().to_string();

        let pre_hash: String = format!("{}{}{}{}{}", sig_path, request, params, nonce, body);

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

    fn post_r_request_headers(
        &self,
        request: &str,
        params: &str,
        body: &str,
    ) -> Result<HeaderMap, BitfinexClientError> {
        self.post_request_headers(REST_API_R_SIGNATURE_PATH, request, params, body)
    }

    fn post_w_request_headers(
        &self,
        request: &str,
        params: &str,
        body: &str,
    ) -> Result<HeaderMap, BitfinexClientError> {
        self.post_request_headers(REST_API_W_SIGNATURE_PATH, request, params, body)
    }
}
