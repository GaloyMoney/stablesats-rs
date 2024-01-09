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
use serde_json::{json, Value};
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

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct BitfinexConfig {
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
    config: BitfinexConfig,
}

impl BitfinexClient {
    pub async fn new(config: BitfinexConfig) -> Result<Self, BitfinexClientError> {
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

    pub fn is_simulated(&self) -> bool {
        self.config.simulated
    }

    pub async fn get_last_price_in_usd_cents(&self) -> Result<LastPrice, BitfinexClientError> {
        let endpoint = "/ticker";
        let mut params = format!("/{}", Instrument::BtcUsdSwap);
        if self.config.simulated {
            params = format!("/{}", Instrument::TestBtcUsdSwap);
        }

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

    #[instrument(name = "bitfinex_client.funding_info", skip(self), err)]
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

    // order_details
    #[instrument(name = "bitfinex_client.get_orders", skip(self), err)]
    pub async fn get_orders(&self) -> Result<Vec<OrderDetails>, BitfinexClientError> {
        let body: HashMap<String, String> = HashMap::new();
        let request_body = serde_json::to_string(&body)?;

        let endpoint = "/orders";
        let mut params = format!("/{}/hist", Instrument::BtcUsdSwap);
        if self.is_simulated() {
            params = format!("/{}/hist", Instrument::TestBtcUsdSwap);
        }
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
        for details in &mut orders {
            if let Some(status) = details.order_status.clone() {
                if status == "EXECUTED" || status == "CANCELED" {
                    details.complete = true;
                }
            }
        }
        Ok(orders)
    }

    async fn get_wallets(&self) -> Result<Vec<WalletDetails>, BitfinexClientError> {
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

    #[instrument(name = "bitfinex_client.funding_account_balance", skip(self), err)]
    pub async fn funding_account_balance(&self) -> Result<AvailableBalance, BitfinexClientError> {
        let currency = if self.is_simulated() {
            Currency::TESTUSDT
        } else {
            Currency::UST
        }
        .to_string();
        let wallets = self.get_wallets().await?;
        let funding_btc_wallet = wallets
            .iter()
            .find(|w| w.wallet_type == "exchange" && w.currency == currency);
        let mut total_amt_in_usdt = Decimal::ZERO;
        let mut free_amt_in_usdt = Decimal::ZERO;
        if let Some(wallet) = funding_btc_wallet {
            total_amt_in_usdt = wallet.balance;
            free_amt_in_usdt = wallet.balance_available;
        }

        Ok(AvailableBalance {
            free_amt_in_usdt,
            total_amt_in_usdt,
            used_amt_in_usdt: total_amt_in_usdt - free_amt_in_usdt,
        })
    }

    #[instrument(name = "bitfinex_client.trading_account_balance", skip(self), err)]
    pub async fn trading_account_balance(&self) -> Result<AvailableBalance, BitfinexClientError> {
        let currency = if self.is_simulated() {
            Currency::TESTUSDTF0
        } else {
            Currency::USTF0
        }
        .to_string();
        let wallets = self.get_wallets().await?;
        let trading_btc_wallet = wallets
            .iter()
            .find(|w| w.wallet_type == "margin" && w.currency == currency);
        let mut total_amt_in_usdt = Decimal::ZERO;
        let mut free_amt_in_usdt = Decimal::ZERO;
        if let Some(wallet) = trading_btc_wallet {
            total_amt_in_usdt = wallet.balance;
            free_amt_in_usdt = wallet.balance_available;
        }

        Ok(AvailableBalance {
            free_amt_in_usdt,
            total_amt_in_usdt,
            used_amt_in_usdt: total_amt_in_usdt - free_amt_in_usdt,
        })
    }

    #[instrument(name = "bitfinex_client.get_positions", skip(self), err)]
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

    #[instrument(name = "bitfinex_client.close_position", skip(self), err)]
    pub async fn close_position(&self) -> Result<SubmittedOrderDetails, BitfinexClientError> {
        let positions = self.get_positions().await?;

        let symbol = if self.is_simulated() {
            Instrument::TestBtcUsdSwap
        } else {
            Instrument::BtcUsdSwap
        }
        .to_string();
        let btc_usd_swap_position = positions
            .into_iter()
            .find(|p| p.status == "ACTIVE" && p.symbol == symbol);
        if let Some(position) = btc_usd_swap_position {
            let response = self
                .submit_order(
                    ClientId::new(),
                    position.amount * Decimal::from(-1),
                    position.leverage,
                )
                .await?;

            return Ok(response);
        }
        Err(BitfinexClientError::UnexpectedResponse {
            msg: format!("No active position found for {}", symbol),
            code: 0,
        })
    }

    #[instrument(name = "bitfinex_client.get_funding_deposit_address", skip(self), err)]
    pub async fn get_funding_deposit_address(&self) -> Result<DepositAddress, BitfinexClientError> {
        let mut body: HashMap<String, String> = HashMap::new();
        body.insert("wallet".to_string(), Wallet::EXCHANGE.to_string());
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

    #[instrument(name = "bitfinex_client.transfer_funding_to_trading", skip(self), err)]
    pub async fn transfer_funding_to_trading(
        &self,
        client_id: ClientId,
        amount: Decimal,
    ) -> Result<TransferDetails, BitfinexClientError> {
        let mut body: HashMap<String, String> = HashMap::new();
        body.insert("from".to_string(), Wallet::EXCHANGE.to_string());
        body.insert("to".to_string(), Wallet::MARGIN.to_string());
        if self.config.simulated {
            body.insert("currency".to_string(), Currency::TESTUSDT.to_string());
            body.insert("currency_to".to_string(), Currency::TESTUSDTF0.to_string());
        } else {
            body.insert("currency".to_string(), Currency::UST.to_string());
            body.insert("currency_to".to_string(), Currency::USTF0.to_string());
        }
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
        Ok(transfer)
    }

    #[instrument(name = "bitfinex_client.transfer_trading_to_funding", skip(self), err)]
    pub async fn transfer_trading_to_funding(
        &self,
        client_id: ClientId,
        amount: Decimal,
    ) -> Result<TransferDetails, BitfinexClientError> {
        let mut body: HashMap<String, String> = HashMap::new();
        body.insert("from".to_string(), Wallet::MARGIN.to_string());
        body.insert("to".to_string(), Wallet::EXCHANGE.to_string());
        if self.config.simulated {
            body.insert("currency".to_string(), Currency::TESTUSDTF0.to_string());
            body.insert("currency_to".to_string(), Currency::TESTUSDT.to_string());
        } else {
            body.insert("currency".to_string(), Currency::USTF0.to_string());
            body.insert("currency_to".to_string(), Currency::UST.to_string());
        }
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
        Ok(transfer)
    }

    #[instrument(name = "bitfinex_client.withdraw_btc_onchain", skip(self), err)]
    pub async fn withdraw_btc_onchain(
        &self,
        client_id: ClientId,
        amount: Decimal,
        fee: Decimal,
        address: String,
    ) -> Result<WithdrawDetails, BitfinexClientError> {
        let mut body: HashMap<String, String> = HashMap::new();
        body.insert("wallet".to_string(), Wallet::EXCHANGE.to_string());
        body.insert("method".to_string(), AddressMethod::BITCOIN.to_string());
        body.insert("amount".to_string(), amount.to_string());
        body.insert("address".to_string(), address.to_string());
        body.insert("payment_id".to_string(), client_id.0.to_string());
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
        Ok(transfer)
    }

    #[instrument(
        name = "bitfinex_client.get_btc_on_chain_transactions",
        skip(self),
        err
    )]
    pub async fn get_btc_on_chain_transactions(
        &self,
    ) -> Result<Vec<Transaction>, BitfinexClientError> {
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

        let transactions = Self::extract_response_data::<Vec<Transaction>>(response).await?;

        Ok(transactions)
    }

    pub async fn fetch_deposit(
        &self,
        depo_addr: String,
        amt_in_btc: Decimal,
    ) -> Result<Transaction, BitfinexClientError> {
        let transactions = self.get_btc_on_chain_transactions().await?;
        let deposit = transactions.into_iter().find(|deposit_entry| {
            deposit_entry.destination_address == depo_addr && deposit_entry.amount == amt_in_btc
        });
        if let Some(deposit_data) = deposit {
            tracing::Span::current().record("deposit_found", true);
            tracing::Span::current().record("bitfinex_deposit_state", &deposit_data.status);
            return Ok(deposit_data);
        }
        Err(BitfinexClientError::UnexpectedResponse {
            msg: format!("No deposit of {amt_in_btc} made to {depo_addr}"),
            code: 0,
        })
    }

    #[instrument(name = "bitfinex_client.submit_order", skip(self), err)]
    pub async fn submit_order(
        &self,
        client_id: ClientId,
        amount: Decimal,
        leverage: Decimal,
    ) -> Result<SubmittedOrderDetails, BitfinexClientError> {
        let mut body: HashMap<String, Value> = HashMap::new();
        body.insert("cid".to_string(), json!(client_id.0));
        body.insert("type".to_string(), json!(OrderType::MARKET.to_string()));
        let instrument = if self.is_simulated() {
            Instrument::TestBtcUsdSwap
        } else {
            Instrument::BtcUsdSwap
        }
        .to_string();
        body.insert("symbol".to_string(), json!(instrument));
        body.insert("amount".to_string(), json!((amount.to_string())));
        body.insert("lev".to_string(), json!((leverage)));
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

    #[instrument(name = "bitfinex_client.get_api_key_permissions", skip(self), err)]
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
        format!("{REST_API_V2_URL}{endpoint}{params}")
    }

    fn url_for_auth_r_path(endpoint: &str, params: &str) -> String {
        format!("{REST_API_V2_URL}/auth/r{endpoint}{params}")
    }

    fn url_for_auth_w_path(endpoint: &str, params: &str) -> String {
        format!("{REST_API_V2_URL}/auth/w{endpoint}{params}")
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

        let pre_hash: String = format!("{sig_path}{request}{params}{nonce}{body}");

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
