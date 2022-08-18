mod error;
mod okex_response;

use std::{collections::HashMap, time::Duration};

use chrono::{SecondsFormat, Utc};
use data_encoding::BASE64;
use ring::hmac;
use rust_decimal::Decimal;

use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use reqwest::Client as ReqwestClient;

pub use error::*;
use okex_response::*;

use governor::{
    clock::DefaultClock, state::keyed::DefaultKeyedStateStore, Jitter, Quota, RateLimiter,
};
use std::num::NonZeroU32;

lazy_static::lazy_static! {
    static ref LIMITER: RateLimiter<&'static str, DefaultKeyedStateStore<&'static str>, DefaultClock>  = RateLimiter::keyed(Quota::per_second(NonZeroU32::new(1).unwrap()));
}

const OKEX_API_URL: &str = "https://www.okex.com";
pub const OKEX_MINIMUM_WITHDRAWAL_AMOUNT: &str = "0.001";
pub const OKEX_MINIMUM_WITHDRAWAL_FEE: &str = "0.0002";

#[derive(Debug, PartialEq, Eq)]
pub struct DepositAddress {
    pub value: String,
}

#[derive(Debug)]
pub struct TransferId {
    pub value: String,
}

#[derive(Debug)]
pub struct AvailableBalance {
    pub amt_in_btc: Decimal,
}

#[derive(Debug)]
pub struct TransferState {
    pub value: String,
}

#[derive(Debug)]
pub struct WithdrawId {
    pub value: String,
}

#[derive(Debug)]
pub struct DepositStatus {
    pub status: String,
}

#[derive(Debug)]
pub struct OrderId {
    pub value: String,
}

#[derive(Debug)]
pub struct PositionId {
    pub value: String,
}

#[derive(Debug, Clone)]
pub enum OkexInstrumentId {
    BtcUsdSwap(String),
    BtcUsd(String),
}

impl OkexInstrumentId {
    pub fn swap() -> Self {
        Self::BtcUsdSwap("BTC-USD-SWAP".to_string())
    }

    pub fn spot() -> Self {
        Self::BtcUsd("BTC-USD".to_string())
    }
}

#[derive(Debug, Clone)]
pub enum OkexMarginMode {
    Cross(String),
    Isolated(String),
}

impl OkexMarginMode {
    pub fn cross() -> Self {
        Self::Cross("cross".to_string())
    }

    pub fn isolated() -> Self {
        Self::Isolated("isolated".to_string())
    }
}

#[derive(Debug, Clone)]
pub enum OkexPositionSide {
    LongShort(String),
    Net(String),
}

impl OkexPositionSide {
    pub fn long() -> Self {
        Self::LongShort("long".to_string())
    }

    pub fn short() -> Self {
        Self::LongShort("short".to_string())
    }

    pub fn net() -> Self {
        Self::Net("net".to_string())
    }
}

#[derive(Debug, Clone)]
pub enum OkexOrderSide {
    Buy,
    Sell,
}

pub struct OkexClientConfig {
    pub api_key: String,
    pub passphrase: String,
    pub secret_key: String,
    pub simulated: String,
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

    pub async fn rate_limit_client(&self, key: &'static str) -> &ReqwestClient {
        let jitter = Jitter::new(Duration::from_secs(1), Duration::from_secs(1));
        LIMITER.until_key_ready_with_jitter(&key, jitter).await;
        &self.client
    }

    pub async fn get_funding_deposit_address(&self) -> Result<DepositAddress, OkexClientError> {
        let request_path = "/api/v5/asset/deposit-address?ccy=BTC";

        let headers = self.get_request_headers(request_path)?;

        let response = self
            .rate_limit_client(request_path)
            .await
            .get(Self::url_for_path(request_path))
            .headers(headers)
            .send()
            .await?;

        let addr_data = Self::extract_response_data::<DepositAddressData>(response).await?;
        Ok(DepositAddress {
            value: addr_data.addr,
        })
    }

    pub async fn transfer_funding_to_trading(
        &self,
        amt: Decimal,
    ) -> Result<TransferId, OkexClientError> {
        let mut body: HashMap<String, String> = HashMap::new();
        body.insert("ccy".to_string(), "BTC".to_string());
        body.insert("amt".to_string(), amt.to_string());
        body.insert("from".to_string(), "6".to_string());
        body.insert("to".to_string(), "18".to_string());
        let request_body = serde_json::to_string(&body)?;

        let request_path = "/api/v5/asset/transfer";
        let headers = self.post_request_headers(request_path, request_body.as_str())?;

        let response = self
            .rate_limit_client(request_path)
            .await
            .post(Self::url_for_path(request_path))
            .headers(headers)
            .body(request_body)
            .send()
            .await?;

        let transfer_data = Self::extract_response_data::<TransferData>(response).await?;
        Ok(TransferId {
            value: transfer_data.trans_id,
        })
    }

    pub async fn transfer_trading_to_funding(
        &self,
        amt: Decimal,
    ) -> Result<TransferId, OkexClientError> {
        let mut body: HashMap<String, String> = HashMap::new();
        body.insert("ccy".to_string(), "BTC".to_string());
        body.insert("amt".to_string(), amt.to_string());
        body.insert("from".to_string(), "18".to_string());
        body.insert("to".to_string(), "6".to_string());
        let request_body = serde_json::to_string(&body)?;

        let request_path = "/api/v5/asset/transfer";
        LIMITER.until_key_ready(&request_path).await;
        let headers = self.post_request_headers(request_path, request_body.as_str())?;

        let response = self
            .rate_limit_client(request_path)
            .await
            .post(Self::url_for_path(request_path))
            .headers(headers)
            .body(request_body)
            .send()
            .await?;

        let transfer_data = Self::extract_response_data::<TransferData>(response).await?;
        Ok(TransferId {
            value: transfer_data.trans_id,
        })
    }

    pub async fn funding_account_balance(&self) -> Result<AvailableBalance, OkexClientError> {
        let request_path = "/api/v5/asset/balances?ccy=BTC";

        let headers = self.get_request_headers(request_path)?;

        let response = self
            .rate_limit_client(request_path)
            .await
            .get(Self::url_for_path(request_path))
            .headers(headers)
            .send()
            .await?;

        let funding_balance = Self::extract_response_data::<FundingBalanceData>(response).await?;
        let balance = Decimal::from_str_exact(&funding_balance.avail_bal)?;

        Ok(AvailableBalance {
            amt_in_btc: balance,
        })
    }

    pub async fn trading_account_balance(&self) -> Result<AvailableBalance, OkexClientError> {
        let request_path = "/api/v5/account/balance?ccy=BTC";

        let headers = self.get_request_headers(request_path)?;

        let response = self
            .rate_limit_client(request_path)
            .await
            .get(Self::url_for_path(request_path))
            .headers(headers)
            .send()
            .await?;

        let trading_balance = Self::extract_response_data::<TradingBalanceData>(response).await?;

        Ok(AvailableBalance {
            amt_in_btc: trading_balance.details[0].cash_bal,
        })
    }

    pub async fn transfer_state(
        &self,
        transfer_id: TransferId,
    ) -> Result<TransferState, OkexClientError> {
        let static_request_path = "/api/v5/asset/transfer-state?ccy=BTC&transId=";
        let request_path = format!("{}{}", static_request_path, transfer_id.value);

        let headers = self.get_request_headers(&request_path)?;

        let response = self
            .rate_limit_client(static_request_path)
            .await
            .get(Self::url_for_path(&request_path))
            .headers(headers)
            .send()
            .await?;

        let state_data = Self::extract_response_data::<TransferStateData>(response).await?;

        Ok(TransferState {
            value: state_data.state,
        })
    }

    pub async fn withdraw_btc_onchain(
        &self,
        amt: Decimal,
        fee: Decimal,
        btc_address: String,
    ) -> Result<WithdrawId, OkexClientError> {
        let mut body: HashMap<String, String> = HashMap::new();
        body.insert("ccy".to_string(), "BTC".to_string());
        body.insert("amt".to_string(), amt.to_string());
        body.insert("dest".to_string(), "4".to_string());
        body.insert("fee".to_string(), fee.to_string());
        body.insert("chain".to_string(), "BTC-Bitcoin".to_string());
        body.insert("toAddr".to_string(), btc_address);
        let request_body = serde_json::to_string(&body)?;

        let request_path = "/api/v5/asset/withdrawal";
        let headers = self.post_request_headers(request_path, request_body.as_str())?;

        let response = self
            .rate_limit_client(request_path)
            .await
            .post(Self::url_for_path(request_path))
            .headers(headers)
            .body(request_body)
            .send()
            .await?;

        let withdraw_data = Self::extract_response_data::<WithdrawData>(response).await?;

        Ok(WithdrawId {
            value: withdraw_data.wd_id,
        })
    }

    pub async fn fetch_deposit(
        &self,
        depo_addr: String,
        amt_in_btc: Decimal,
    ) -> Result<DepositStatus, OkexClientError> {
        // 1. Get all deposit history
        let request_path = "/api/v5/asset/deposit-history";
        let headers = self.get_request_headers(request_path)?;
        let response = self
            .rate_limit_client(request_path)
            .await
            .get(Self::url_for_path(request_path))
            .headers(headers)
            .send()
            .await?;

        let history = Self::extract_response_data_array::<DepositHistoryData>(response).await?;

        // 2. Filter through results from above and find any entry that matches addr and amt_in_btc
        let deposit = history.into_iter().find(|deposit_entry| {
            deposit_entry.to == depo_addr && deposit_entry.amt == amt_in_btc.to_string()
        });

        if let Some(deposit_data) = deposit {
            Ok(DepositStatus {
                status: deposit_data.state,
            })
        } else {
            Err(OkexClientError::UnexpectedResponse {
                msg: format!("No deposit of {} made to {}", amt_in_btc, depo_addr),
                code: "0".to_string(),
            })
        }
    }

    /// Open a hedging position on an instrument
    ///
    /// Parameters:
    ///     order_type(String): "market"
    ///     size(u64): e.g. 20
    pub async fn place_order(
        &self,
        inst_id: OkexInstrumentId,
        margin_mode: OkexMarginMode,
        side: OkexOrderSide,
        pos_side: OkexPositionSide,
        order_type: String,
        size: u64,
    ) -> Result<OrderId, OkexClientError> {
        let instrument = match inst_id {
            OkexInstrumentId::BtcUsdSwap(inst) => inst,
            OkexInstrumentId::BtcUsd(inst) => inst,
        };
        let margin = match margin_mode {
            OkexMarginMode::Cross(margin) => margin,
            OkexMarginMode::Isolated(margin) => margin,
        };
        let pos_side = match pos_side {
            OkexPositionSide::LongShort(pos_side) => pos_side,
            OkexPositionSide::Net(pos_side) => pos_side,
        };
        let order_side = match side {
            OkexOrderSide::Buy => "buy".to_string(),
            OkexOrderSide::Sell => "sell".to_string(),
        };

        let mut body: HashMap<String, String> = HashMap::new();
        body.insert("ccy".to_string(), "BTC".to_string());
        body.insert("instId".to_string(), instrument);
        body.insert("tdMode".to_string(), margin);
        body.insert("side".to_string(), order_side);
        body.insert("ordType".to_string(), order_type);
        body.insert("posSide".to_string(), pos_side);
        body.insert("sz".to_string(), size.to_string());
        let request_body = serde_json::to_string(&body)?;

        let request_path = "/api/v5/trade/order";
        let headers = self.post_request_headers(request_path, &request_body)?;

        let response = self
            .rate_limit_client(request_path)
            .await
            .post(Self::url_for_path(request_path))
            .headers(headers)
            .body(request_body)
            .send()
            .await?;

        let order_data = Self::extract_response_data::<OrderData>(response).await?;
        Ok(OrderId {
            value: order_data.ord_id,
        })
    }

    pub async fn get_position(&self) -> Result<PositionId, OkexClientError> {
        let request_path = "/api/v5/account/positions?instId=BTC-USD-SWAP";
        let headers = self.get_request_headers(request_path)?;

        let response = self
            .rate_limit_client(request_path)
            .await
            .get(Self::url_for_path(request_path))
            .headers(headers)
            .send()
            .await?;

        let position_data = Self::extract_response_data::<PositionData>(response).await?;

        Ok(PositionId {
            value: position_data.pos_id,
        })
    }

    pub async fn close_positions(
        &self,
        inst_id: OkexInstrumentId,
        pos_side: OkexPositionSide,
        margin_mode: OkexMarginMode,
        ccy: String,
        auto_cxl: bool,
    ) -> Result<ClosePositionData, OkexClientError> {
        let instrument = match inst_id {
            OkexInstrumentId::BtcUsdSwap(inst) => inst,
            OkexInstrumentId::BtcUsd(inst) => inst,
        };
        let margin = match margin_mode {
            OkexMarginMode::Cross(margin) => margin,
            OkexMarginMode::Isolated(margin) => margin,
        };
        let pos_side = match pos_side {
            OkexPositionSide::LongShort(pos_side) => pos_side,
            OkexPositionSide::Net(pos_side) => pos_side,
        };

        let mut body: HashMap<String, String> = HashMap::new();
        body.insert("instId".to_string(), instrument);
        body.insert("mgnMode".to_string(), margin);
        body.insert("posSide".to_string(), pos_side);
        body.insert("ccy".to_string(), ccy);
        body.insert("autoCxl".to_string(), auto_cxl.to_string());
        let request_body = serde_json::to_string(&body)?;

        let request_path = "/api/v5/trade/close-position";
        let headers = self.post_request_headers(request_path, &request_body)?;

        let response = self
            .rate_limit_client(request_path)
            .await
            .post(Self::url_for_path(request_path))
            .headers(headers)
            .body(request_body)
            .send()
            .await?;

        let close_position = Self::extract_response_data::<ClosePositionData>(response).await?;

        Ok(close_position)
    }

    /// Extracts the first entry in the response data
    async fn extract_response_data<T: serde::de::DeserializeOwned>(
        response: reqwest::Response,
    ) -> Result<T, OkexClientError> {
        let response_text = response.text().await?;
        let OkexResponse { code, msg, data } =
            serde_json::from_str::<OkexResponse<T>>(&response_text)?;
        if let Some(data) = data {
            if let Some(first) = data.into_iter().next() {
                return Ok(first);
            }
        }
        Err(OkexClientError::UnexpectedResponse { msg, code })
    }

    /// Extracts the array of entries in the response data
    async fn extract_response_data_array<T: serde::de::DeserializeOwned>(
        response: reqwest::Response,
    ) -> Result<Vec<T>, OkexClientError> {
        let response_text = response.text().await?;
        let OkexResponse { code, msg, data } =
            serde_json::from_str::<OkexResponse<T>>(&response_text)?;

        if let Some(data) = data {
            return Ok(data);
        }
        Err(OkexClientError::UnexpectedResponse { msg, code })
    }

    fn sign_okex_request(&self, pre_hash: String) -> String {
        let key = hmac::Key::new(hmac::HMAC_SHA256, self.config.secret_key.as_bytes());
        let signature = hmac::sign(&key, pre_hash.as_bytes());
        BASE64.encode(signature.as_ref())
    }

    fn url_for_path(path: &str) -> String {
        format!("{}{}", OKEX_API_URL, path)
    }

    fn post_request_headers(
        &self,
        request_path: &str,
        request_body: &str,
    ) -> Result<HeaderMap, OkexClientError> {
        let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        let pre_hash = format!("{}POST{}{}", timestamp, request_path, request_body);
        self.request_headers(timestamp, pre_hash)
    }

    fn get_request_headers(&self, request_path: &str) -> Result<HeaderMap, OkexClientError> {
        let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        let pre_hash = format!("{}GET{}", timestamp, request_path);
        self.request_headers(timestamp, pre_hash)
    }

    #[allow(clippy::or_fun_call)]
    fn request_headers(
        &self,
        formatted_timestamp: String,
        pre_hash: String,
    ) -> Result<HeaderMap, OkexClientError> {
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
            HeaderValue::from_str(formatted_timestamp.as_str())?,
        );
        headers.insert(
            "OK-ACCESS-PASSPHRASE",
            HeaderValue::from_str(self.config.passphrase.as_str())?,
        );
        headers.insert(
            "x-simulated-trading",
            HeaderValue::from_str(self.config.simulated.as_str())?,
        );

        Ok(headers)
    }
}
