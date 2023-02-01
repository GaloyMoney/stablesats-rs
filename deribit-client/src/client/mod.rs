mod deribit_response;
mod error;
mod primitives;

use data_encoding::BASE64;
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    Client as ReqwestClient, Response, StatusCode,
};
use rust_decimal::Decimal;
use shared::exchanges_config::DeribitConfig;
use tracing::instrument;

use std::time::Duration;

use deribit_response::*;
pub use error::*;
pub use primitives::*;

use governor::{
    clock::DefaultClock, state::keyed::DefaultKeyedStateStore, Jitter, Quota, RateLimiter,
};
use std::num::NonZeroU32;

lazy_static::lazy_static! {
    static ref LIMITER: RateLimiter<&'static str, DefaultKeyedStateStore<&'static str>, DefaultClock>  = RateLimiter::keyed(Quota::per_second(NonZeroU32::new(1).unwrap()));
}

const REST_API_V2_URL: &str = "https://www.deribit.com/api/v2";
const TEST_REST_API_V2_URL: &str = "https://test.deribit.com/api/v2";

#[derive(Clone)]
pub struct DeribitClient {
    client: ReqwestClient,
    config: DeribitConfig,
}

impl DeribitClient {
    pub async fn new(config: DeribitConfig) -> Result<Self, DeribitClientError> {
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

    pub async fn get_last_price_in_usd_cents(&self) -> Result<LastPrice, DeribitClientError> {
        let endpoint = "/public/ticker";
        let params = format!("?instrument_name={}", Instrument::BtcUsdSwap);

        let response = self
            .rate_limit_client(endpoint)
            .await
            .get(self.url_for_path(endpoint, params.as_str()))
            .send()
            .await?;

        let details = Self::extract_response_data::<LastPriceDataDetails>(response).await?;
        Ok(LastPrice {
            usd_cents: details.result.last_price * Decimal::ONE_HUNDRED,
        })
    }

    #[instrument(skip(self), err)]
    pub async fn get_btc_on_chain_deposit_address(
        &self,
    ) -> Result<DepositAddressData, DeribitClientError> {
        let endpoint = "/private/get_current_deposit_address";
        let params = format!("?currency={}", Currency::BTC);

        let headers = self.get_private_request_headers(KeyUsage::ForFunding)?;

        let response = self
            .rate_limit_client(endpoint)
            .await
            .get(self.url_for_path(endpoint, params.as_str()))
            .headers(headers)
            .send()
            .await?;

        let details = Self::extract_response_data::<DepositAddressDetails>(response).await?;

        if let Some(data) = details.result {
            Ok(data)
        } else {
            Err(DeribitClientError::UnexpectedResponse {
                msg: "No deposit address returned".to_string(),
                code: 0,
            })
        }
    }

    #[instrument(skip(self), err)]
    pub async fn get_deposits(&self) -> Result<Vec<Deposit>, DeribitClientError> {
        let endpoint = "/private/get_deposits";
        let params = format!("?currency={}", Currency::BTC);

        let headers = self.get_private_request_headers(KeyUsage::ForFunding)?;

        let response = self
            .rate_limit_client(endpoint)
            .await
            .get(self.url_for_path(endpoint, params.as_str()))
            .headers(headers)
            .send()
            .await?;

        let details = Self::extract_response_data::<DepositDetails>(response).await?;

        Ok(details.result.data)
    }

    #[instrument(skip(self), err)]
    pub async fn get_transfers(&self) -> Result<Vec<Transfer>, DeribitClientError> {
        let endpoint = "/private/get_transfers";
        let params = format!("?currency={}", Currency::BTC);

        let headers = self.get_private_request_headers(KeyUsage::ForFunding)?;

        let response = self
            .rate_limit_client(endpoint)
            .await
            .get(self.url_for_path(endpoint, params.as_str()))
            .headers(headers)
            .send()
            .await?;

        let details = Self::extract_response_data::<TransferDetails>(response).await?;

        Ok(details.result.data)
    }

    #[instrument(skip(self), err)]
    pub async fn get_withdrawals(&self) -> Result<Vec<Transfer>, DeribitClientError> {
        let endpoint = "/private/get_withdrawals";
        let params = format!("?currency={}", Currency::BTC);

        let headers = self.get_private_request_headers(KeyUsage::ForFunding)?;

        let response = self
            .rate_limit_client(endpoint)
            .await
            .get(self.url_for_path(endpoint, params.as_str()))
            .headers(headers)
            .send()
            .await?;

        let details = Self::extract_response_data::<TransferDetails>(response).await?;

        Ok(details.result.data)
    }

    #[instrument(skip(self), err)]
    pub async fn withdraw(
        &self,
        client_id: ClientId,
        amount: Decimal,
        fee: Decimal,
        btc_address: String,
    ) -> Result<Vec<Transfer>, DeribitClientError> {
        let endpoint = "/private/withdraw";
        let params = format!(
            "?currency={}&address={}&amount={}&priority={}",
            Currency::BTC,
            btc_address,
            amount,
            Priority::VeryHigh,
        );

        let headers = self.get_private_request_headers(KeyUsage::ForFunding)?;

        let response = self
            .rate_limit_client(endpoint)
            .await
            .get(self.url_for_path(endpoint, params.as_str()))
            .headers(headers)
            .send()
            .await?;

        let details = Self::extract_response_data::<TransferDetails>(response).await?;

        Ok(details.result.data)
    }

    #[instrument(skip(self), err)]
    pub async fn buy(
        &self,
        client_id: ClientId,
        amount_in_usd: Decimal,
    ) -> Result<Order, DeribitClientError> {
        let endpoint = "/private/buy";
        let params = format!(
            "?instrument_name={}&amount={}&type=market&label={}",
            Instrument::BtcUsdSwap,
            amount_in_usd,
            client_id.0,
        );

        let headers = self.get_private_request_headers(KeyUsage::ForTrading)?;

        let response = self
            .rate_limit_client(endpoint)
            .await
            .get(self.url_for_path(endpoint, params.as_str()))
            .headers(headers)
            .send()
            .await?;

        let details = Self::extract_response_data::<OrderDetails>(response).await?;

        Ok(details.result.order)
    }

    #[instrument(skip(self), err)]
    pub async fn sell(
        &self,
        client_id: ClientId,
        amount_in_usd: Decimal,
    ) -> Result<Order, DeribitClientError> {
        let endpoint = "/private/sell";
        let params = format!(
            "?instrument_name={}&amount={}&type=market&label={}",
            Instrument::BtcUsdSwap,
            amount_in_usd,
            client_id.0,
        );

        let headers = self.get_private_request_headers(KeyUsage::ForTrading)?;

        let response = self
            .rate_limit_client(endpoint)
            .await
            .get(self.url_for_path(endpoint, params.as_str()))
            .headers(headers)
            .send()
            .await?;

        let details = Self::extract_response_data::<OrderDetails>(response).await?;

        Ok(details.result.order)
    }

    #[instrument(skip(self), err)]
    pub async fn close_position(&self, client_id: ClientId) -> Result<Order, DeribitClientError> {
        let endpoint = "/private/close_position";
        let params = format!(
            "?instrument_name={}&type=market&label={}",
            Instrument::BtcUsdSwap,
            client_id.0,
        );

        let headers = self.get_private_request_headers(KeyUsage::ForTrading)?;

        let response = self
            .rate_limit_client(endpoint)
            .await
            .get(self.url_for_path(endpoint, params.as_str()))
            .headers(headers)
            .send()
            .await?;

        let details = Self::extract_response_data::<OrderDetails>(response).await?;

        Ok(details.result.order)
    }

    #[instrument(skip(self), err)]
    pub async fn get_order_state(&self, order_id: String) -> Result<Order, DeribitClientError> {
        let endpoint = "/private/get_order_state";
        let params = format!("?order_id={order_id}");

        let headers = self.get_private_request_headers(KeyUsage::ForTrading)?;

        let response = self
            .rate_limit_client(endpoint)
            .await
            .get(self.url_for_path(endpoint, params.as_str()))
            .headers(headers)
            .send()
            .await?;

        let details = Self::extract_response_data::<OrderStateDetails>(response).await?;

        Ok(details.result)
    }

    #[instrument(skip(self), err)]
    pub async fn get_position(&self) -> Result<Position, DeribitClientError> {
        let endpoint = "/private/get_position";
        let params = format!("?instrument_name={}", Instrument::BtcUsdSwap,);

        let headers = self.get_private_request_headers(KeyUsage::ForTrading)?;

        let response = self
            .rate_limit_client(endpoint)
            .await
            .get(self.url_for_path(endpoint, params.as_str()))
            .headers(headers)
            .send()
            .await?;

        let details = Self::extract_response_data::<PositionDetails>(response).await?;

        Ok(details.result)
    }

    #[instrument(skip(self), err)]
    pub async fn get_funding_account_summary(&self) -> Result<AccountSummary, DeribitClientError> {
        let endpoint = "/private/get_account_summary";
        let params = format!("?currency={}", Currency::BTC,);

        let headers = self.get_private_request_headers(KeyUsage::ForFunding)?;

        let response = self
            .rate_limit_client(endpoint)
            .await
            .get(self.url_for_path(endpoint, params.as_str()))
            .headers(headers)
            .send()
            .await?;

        let details = Self::extract_response_data::<AccountSummaryDetails>(response).await?;

        Ok(details.result)
    }

    #[instrument(skip(self), err)]
    pub async fn get_trading_account_summary(&self) -> Result<AccountSummary, DeribitClientError> {
        let endpoint = "/private/get_account_summary";
        let params = format!("?currency={}", Currency::BTC,);

        let headers = self.get_private_request_headers(KeyUsage::ForTrading)?;

        let response = self
            .rate_limit_client(endpoint)
            .await
            .get(self.url_for_path(endpoint, params.as_str()))
            .headers(headers)
            .send()
            .await?;

        let details = Self::extract_response_data::<AccountSummaryDetails>(response).await?;

        Ok(details.result)
    }

    async fn extract_response_data<T: serde::de::DeserializeOwned>(
        response: Response,
    ) -> Result<T, DeribitClientError> {
        match response.status() {
            StatusCode::OK => {
                let response_text = response.text().await?;
                dbg!(response_text.clone());
                match serde_json::from_str::<T>(&response_text) {
                    Ok(data) => Ok(data),
                    Err(err) => Err(DeribitClientError::UnexpectedResponse {
                        msg: err.to_string(),
                        code: 0,
                    }),
                }
            }
            _ => {
                let response_text = response.text().await?;
                dbg!(response_text.clone());
                let data = serde_json::from_str::<DeribitErrorResponse>(&response_text)?;
                Err(DeribitClientError::from((
                    data.error.message,
                    data.error.code,
                )))
            }
        }
    }

    fn url_for_path(&self, endpoint: &str, params: &str) -> String {
        if self.config.simulated {
            format!("{}{}{}", TEST_REST_API_V2_URL, endpoint, params)
        } else {
            format!("{}{}{}", REST_API_V2_URL, endpoint, params)
        }
    }

    fn get_private_request_headers(
        &self,
        key_usage: KeyUsage,
    ) -> Result<HeaderMap, DeribitClientError> {
        let mut token = format!(
            "{}:{}",
            self.config.funding_api_key, self.config.funding_secret_key
        );
        if key_usage == KeyUsage::ForTrading {
            token = format!(
                "{}:{}",
                self.config.trading_api_key, self.config.trading_secret_key
            );
        }
        let auth = format!("Basic {}", BASE64.encode(token.as_ref()));

        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_str(auth.as_str())?);

        Ok(headers)
    }
}
