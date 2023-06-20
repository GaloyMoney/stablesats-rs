mod error;
mod okex_response;
mod primitives;

use chrono::{SecondsFormat, Utc};
use data_encoding::BASE64;
use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_TYPE},
    Client as ReqwestClient,
};
use ring::hmac;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use std::{collections::HashMap, time::Duration};

pub use error::*;
pub use okex_response::OrderDetails;
pub use okex_response::TransferStateData;
use okex_response::*;
pub use primitives::*;

use governor::{
    clock::DefaultClock, state::keyed::DefaultKeyedStateStore, Jitter, Quota, RateLimiter,
};
use std::num::NonZeroU32;

lazy_static::lazy_static! {
    static ref LIMITER: RateLimiter<&'static str, DefaultKeyedStateStore<&'static str>, DefaultClock>  = RateLimiter::keyed(Quota::per_second(NonZeroU32::new(1).unwrap()));
}

const TESTNET_BURNER_ADDRESS: &str = "tb1qfqh7ksqcrhjgq35clnf06l5d9s6tk2ke46ecrj";
const OKEX_API_URL: &str = "https://www.okex.com";
pub const OKEX_MINIMUM_WITHDRAWAL_FEE: Decimal = dec!(0.0002);
pub const OKEX_MAXIMUM_WITHDRAWAL_FEE: Decimal = dec!(0.0004);
pub const OKEX_MINIMUM_WITHDRAWAL_AMOUNT: Decimal = dec!(0.001);
pub const OKEX_MAXIMUM_WITHDRAWAL_AMOUNT: Decimal = dec!(500);

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct OkexClientConfig {
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub passphrase: String,
    #[serde(default)]
    pub secret_key: String,
    #[serde(default)]
    pub simulated: bool,
}

#[derive(Clone)]
pub struct OkexClient {
    client: ReqwestClient,
    config: OkexClientConfig,
}

impl OkexClient {
    pub async fn new(config: OkexClientConfig) -> Result<Self, OkexClientError> {
        let client = Self {
            client: ReqwestClient::builder().use_rustls_tls().build()?,
            config,
        };
        let path = "/api/v5/account/config";
        let config_url = Self::url_for_path(path);
        let headers = client.get_request_headers(path)?;

        let response = client
            .rate_limit_client(path)
            .await
            .get(config_url)
            .headers(headers)
            .send()
            .await?;
        let config_data =
            Self::extract_response_data::<OkexAccountConfigurationData>(response).await?;

        if &config_data.pos_mode != "net_mode" {
            return Err(OkexClientError::MisconfiguredAccount(format!(
                "Expected `net_mode`, got `{}`",
                config_data.pos_mode
            )));
        }

        if &config_data.acct_lv != "2" {
            return Err(OkexClientError::MisconfiguredAccount(format!(
                "Expected `acct_lv: 2`, got `{}`",
                config_data.acct_lv
            )));
        }
        Ok(client)
    }

    pub async fn check_leverage(&self, expected_leverage: Decimal) -> Result<(), OkexClientError> {
        let leverage_info = self.leverage_info().await?;

        if leverage_info.lever != expected_leverage {
            return Err(OkexClientError::MisconfiguredAccount(format!(
                "Expected `leverage: {}`, got `{}`",
                expected_leverage, leverage_info.lever
            )));
        }
        Ok(())
    }

    pub fn is_simulated(&self) -> bool {
        self.config.simulated
    }

    pub async fn leverage_info(&self) -> Result<OkexLeverageInfoData, OkexClientError> {
        let path = "/api/v5/account/leverage-info?instId=BTC-USD-SWAP&mgnMode=cross";
        let config_url = Self::url_for_path(path);
        let headers = self.get_request_headers(path)?;

        let response = self
            .rate_limit_client(path)
            .await
            .get(config_url)
            .headers(headers)
            .send()
            .await?;
        let leverage_info = Self::extract_response_data::<OkexLeverageInfoData>(response).await?;

        Ok(leverage_info)
    }

    pub async fn rate_limit_client(&self, key: &'static str) -> &ReqwestClient {
        let jitter = Jitter::new(Duration::from_secs(1), Duration::from_secs(1));
        LIMITER.until_key_ready_with_jitter(&key, jitter).await;
        &self.client
    }

    #[instrument(name = "okex_client.get_funding_deposit_address", skip(self), err)]
    pub async fn get_funding_deposit_address(&self) -> Result<DepositAddress, OkexClientError> {
        if self.config.simulated {
            return Ok(DepositAddress {
                value: TESTNET_BURNER_ADDRESS.to_string(),
            });
        }

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

    #[instrument(name = "okex_client.get_onchain_fees", skip(self), err)]
    pub async fn get_onchain_fees(&self) -> Result<OnchainFees, OkexClientError> {
        let request_path = "/api/v5/asset/currencies?ccy=BTC";

        let headers = self.get_request_headers(request_path)?;

        let response = self
            .rate_limit_client(request_path)
            .await
            .get(Self::url_for_path(request_path))
            .headers(headers)
            .send()
            .await?;

        let fees_data_resp = Self::extract_response_data::<OnchainFeesData>(response).await;
        match fees_data_resp {
            Ok(fees_data) => Ok(OnchainFees {
                ccy: fees_data.ccy,
                chain: fees_data.chain,
                min_fee: std::cmp::min(fees_data.min_fee, OKEX_MINIMUM_WITHDRAWAL_FEE),
                max_fee: std::cmp::min(fees_data.max_fee, OKEX_MAXIMUM_WITHDRAWAL_FEE),
                min_withdraw: std::cmp::min(fees_data.min_wd, OKEX_MINIMUM_WITHDRAWAL_AMOUNT),
                max_withdraw: std::cmp::min(fees_data.max_wd, OKEX_MAXIMUM_WITHDRAWAL_AMOUNT),
            }),
            _ => Ok(OnchainFees {
                ccy: "BTC".to_string(),
                chain: "BTC-Bitcoin".to_string(),
                min_fee: OKEX_MINIMUM_WITHDRAWAL_FEE,
                max_fee: OKEX_MAXIMUM_WITHDRAWAL_FEE,
                min_withdraw: OKEX_MINIMUM_WITHDRAWAL_AMOUNT,
                max_withdraw: OKEX_MAXIMUM_WITHDRAWAL_AMOUNT,
            }),
        }
    }

    #[instrument(name = "okex_client.transfer_funding_to_trading", skip(self), err)]
    pub async fn transfer_funding_to_trading(
        &self,
        client_id: ClientTransferId,
        amt: Decimal,
    ) -> Result<TransferId, OkexClientError> {
        let mut body: HashMap<String, String> = HashMap::new();
        body.insert("ccy".to_string(), TradeCurrency::BTC.to_string());
        body.insert("amt".to_string(), amt.to_string());
        body.insert("from".to_string(), "6".to_string());
        body.insert("to".to_string(), "18".to_string());
        body.insert("clientId".to_string(), client_id.0);
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

    #[instrument(name = "okex_client.transfer_trading_to_funding", skip(self), err)]
    pub async fn transfer_trading_to_funding(
        &self,
        client_id: ClientTransferId,
        amt: Decimal,
    ) -> Result<TransferId, OkexClientError> {
        let mut body: HashMap<String, String> = HashMap::new();
        body.insert("ccy".to_string(), TradeCurrency::BTC.to_string());
        body.insert("amt".to_string(), amt.to_string());
        body.insert("from".to_string(), "18".to_string());
        body.insert("to".to_string(), "6".to_string());
        body.insert("clientId".to_string(), client_id.0);
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

    #[instrument(name = "okex_client.funding_account_balance", skip(self), err)]
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

        Ok(AvailableBalance {
            free_amt_in_btc: funding_balance.avail_bal,
            used_amt_in_btc: funding_balance.frozen_bal,
            total_amt_in_btc: funding_balance.bal,
        })
    }

    #[instrument(name = "okex_client.trading_account_balance", skip(self), err)]
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

        let mut free_amt_in_btc = Decimal::ZERO;
        let mut used_amt_in_btc = Decimal::ZERO;
        let mut total_amt_in_btc = Decimal::ZERO;

        if !trading_balance.details.is_empty() {
            free_amt_in_btc = trading_balance.details[0].avail_eq;
            used_amt_in_btc = trading_balance.details[0].frozen_bal;
            total_amt_in_btc = trading_balance.details[0].eq;
        }

        Ok(AvailableBalance {
            free_amt_in_btc,
            used_amt_in_btc,
            total_amt_in_btc,
        })
    }

    #[instrument(name = "okex_client.transfer_state", skip(self), err)]
    pub async fn transfer_state(
        &self,
        transfer_id: TransferId,
    ) -> Result<TransferState, OkexClientError> {
        let static_request_path = "/api/v5/asset/transfer-state?ccy=BTC&transId=";
        let request_path = format!("{static_request_path}{}", transfer_id.value);

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
            state: state_data.state,
            transfer_id: state_data.trans_id,
            client_id: state_data.client_id,
        })
    }

    pub async fn transfer_state_by_client_id(
        &self,
        client_id: ClientTransferId,
    ) -> Result<TransferState, OkexClientError> {
        let static_request_path = "/api/v5/asset/transfer-state?ccy=BTC&clientId=";
        let request_path = format!("{}{}", static_request_path, client_id.0);

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
            state: state_data.state,
            transfer_id: state_data.trans_id,
            client_id: state_data.client_id,
        })
    }

    #[instrument(name = "okex_client.withdraw_btc_onchain", skip(self), err)]
    pub async fn withdraw_btc_onchain(
        &self,
        client_id: ClientTransferId,
        amt: Decimal,
        fee: Decimal,
        btc_address: String,
    ) -> Result<WithdrawId, OkexClientError> {
        let mut body: HashMap<String, String> = HashMap::new();
        body.insert("ccy".to_string(), TradeCurrency::BTC.to_string());
        body.insert("amt".to_string(), amt.to_string());
        body.insert("dest".to_string(), "4".to_string());
        body.insert("fee".to_string(), fee.to_string());
        body.insert("chain".to_string(), "BTC-Bitcoin".to_string());
        body.insert("toAddr".to_string(), btc_address);
        body.insert("clientId".to_string(), client_id.0);
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

    #[instrument(name = "okex_client.fetch_deposit", skip(self), err)]
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
                state: match &deposit_data.state[..] {
                    "0" => "pending".to_string(),  // waiting for confirmation
                    "1" => "success".to_string(),  // deposit credited, cannot withdraw
                    "2" => "success".to_string(),  // deposit successful, can withdraw
                    "8" => "pending".to_string(), // pending due to temporary deposit suspension on this crypto currency
                    "12" => "pending".to_string(), // account or deposit is frozen
                    "13" => "success".to_string(), // sub-account deposit interception
                    _ => "failed".to_string(),
                },
                transaction_id: deposit_data.tx_id,
            })
        } else {
            Err(OkexClientError::UnexpectedResponse {
                msg: format!("No deposit of {amt_in_btc} made to {depo_addr}"),
                code: "0".to_string(),
            })
        }
    }

    #[instrument(name = "okex_client.fetch_withdrawal_by_client_id", skip(self), err)]
    pub async fn fetch_withdrawal_by_client_id(
        &self,
        client_id: ClientTransferId,
    ) -> Result<WithdrawalStatus, OkexClientError> {
        let static_request_path = "/api/v5/asset/withdrawal-history?ccy=BTC&clientId=";
        let request_path = format!("{}{}", static_request_path, client_id.0);
        let headers = self.get_request_headers(&request_path)?;
        let response = self
            .rate_limit_client(static_request_path)
            .await
            .get(Self::url_for_path(&request_path))
            .headers(headers)
            .send()
            .await?;

        let withdrawal_data_option =
            Self::extract_optional_response_data::<WithdrawalHistoryData>(response).await?;

        match withdrawal_data_option {
            Some(withdrawal_data) => {
                Ok(WithdrawalStatus {
                    state: match &withdrawal_data.state[..] {
                        "-3" => "pending".to_string(), // canceling
                        "-2" => "failed".to_string(),  // canceled
                        "-1" => "failed".to_string(),  // failed
                        "0" => "pending".to_string(),  // waiting withdrawal
                        "1" => "pending".to_string(),  // withdrawing
                        "2" => "success".to_string(),  // withdraw success
                        "7" => "pending".to_string(),  // approved
                        "10" => "pending".to_string(), // waiting transfer
                        "4" | "5" | "6" | "8" | "9" | "12" => "pending".to_string(), // waiting manual review
                        _ => "failed".to_string(),
                    },
                    transaction_id: withdrawal_data.tx_id,
                    client_id: withdrawal_data.client_id,
                })
            }
            None => Err(OkexClientError::ParameterClientIdNotFound),
        }
    }

    #[instrument(name = "okex_client.place_order", skip(self), err)]
    pub async fn place_order(
        &self,
        id: ClientOrderId,
        side: OkexOrderSide,
        contracts: &BtcUsdSwapContracts,
    ) -> Result<OrderId, OkexClientError> {
        let mut body: HashMap<String, String> = HashMap::new();
        body.insert("ccy".to_string(), TradeCurrency::BTC.to_string());
        body.insert("clOrdId".to_string(), id.0);
        body.insert(
            "instId".to_string(),
            OkexInstrumentId::BtcUsdSwap.to_string(),
        );
        body.insert("tdMode".to_string(), OkexMarginMode::Cross.to_string());
        body.insert("side".to_string(), side.to_string());
        body.insert("ordType".to_string(), OkexOrderType::Market.to_string());
        body.insert("posSide".to_string(), OkexPositionSide::Net.to_string());
        body.insert("sz".to_string(), contracts.0.to_string());
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
        if order_data.ord_id.is_empty() && !order_data.s_code.is_empty() {
            return Err(OkexClientError::UnexpectedResponse {
                msg: order_data.s_msg,
                code: order_data.s_code,
            });
        }
        Ok(OrderId {
            value: order_data.ord_id,
        })
    }

    #[instrument(name = "okex_client.order_details", skip(self), err)]
    pub async fn order_details(&self, id: ClientOrderId) -> Result<OrderDetails, OkexClientError> {
        let static_request_path = "/api/v5/trade/order?instId=BTC-USD-SWAP&clOrdId=";
        let request_path = format!("{}{}", static_request_path, id.0);
        let headers = self.get_request_headers(&request_path)?;

        let response = self
            .rate_limit_client(static_request_path)
            .await
            .get(Self::url_for_path(&request_path))
            .headers(headers)
            .send()
            .await?;

        let mut details = Self::extract_response_data::<OrderDetails>(response).await?;
        if details.state == "filled" || details.state == "canceled" {
            details.complete = true;
        }
        Ok(details)
    }

    pub async fn get_last_price_in_usd_cents(&self) -> Result<LastPrice, OkexClientError> {
        let request_path = "/api/v5/market/ticker?instId=BTC-USD-SWAP";
        let headers = self.get_request_headers(request_path)?;

        let response = self
            .rate_limit_client(request_path)
            .await
            .get(Self::url_for_path(request_path))
            .headers(headers)
            .send()
            .await?;

        if let Some(LastPriceData { last, .. }) =
            Self::extract_optional_response_data::<LastPriceData>(response).await?
        {
            Ok(LastPrice {
                usd_cents: last * Decimal::ONE_HUNDRED,
            })
        } else {
            Err(OkexClientError::NoLastPriceAvailable)
        }
    }

    #[instrument(
        name = "okex_client.get_position_in_signed_usd_cents",
        skip_all,
        fields(notional_usd, position_in_ct, last_price),
        err
    )]
    pub async fn get_position_in_signed_usd_cents(&self) -> Result<PositionSize, OkexClientError> {
        let request_path = "/api/v5/account/positions?instId=BTC-USD-SWAP";
        let headers = self.get_request_headers(request_path)?;

        let response = self
            .rate_limit_client(request_path)
            .await
            .get(Self::url_for_path(request_path))
            .headers(headers)
            .send()
            .await?;

        if let Some(PositionData {
            notional_usd,
            pos,
            last,
            ..
        }) = Self::extract_optional_response_data::<PositionData>(response).await?
        {
            let span = tracing::Span::current();
            span.record("notional_usd", &tracing::field::display(&notional_usd));
            span.record("position_in_ct", &tracing::field::display(&pos));
            span.record("last_price", &tracing::field::display(&last));

            // Position responses with data:
            //  No position on account: pos = 0 and everything else is empty
            //  Some position on account: pos, notional and last are properly populated
            //  Else: raise an error
            // Position responses without data:
            //  No position on account: successful api call, but no data
            let d_result = pos.parse::<Decimal>();
            let n_result = notional_usd.parse::<Decimal>();
            let l_result = last.parse::<Decimal>();

            match (d_result, n_result, l_result) {
                (Ok(direction), Ok(notional_usd), Ok(last)) => Ok(PositionSize {
                    instrument_id: OkexInstrumentId::BtcUsdSwap,
                    usd_cents: notional_usd
                        * Decimal::ONE_HUNDRED
                        * if direction > Decimal::ZERO {
                            Decimal::ONE
                        } else {
                            Decimal::NEGATIVE_ONE
                        },
                    last_price_in_usd_cents: last * Decimal::ONE_HUNDRED,
                }),
                (Ok(direction), _, _) => {
                    if direction.is_zero() {
                        Ok(PositionSize {
                            instrument_id: OkexInstrumentId::BtcUsdSwap,
                            usd_cents: Decimal::ZERO,
                            last_price_in_usd_cents: Decimal::ZERO,
                        })
                    } else {
                        Err(OkexClientError::NonParsablePositionData)
                    }
                }
                _ => Err(OkexClientError::NonParsablePositionData),
            }
        } else {
            Ok(PositionSize {
                instrument_id: OkexInstrumentId::BtcUsdSwap,
                usd_cents: Decimal::ZERO,
                last_price_in_usd_cents: Decimal::ZERO,
            })
        }
    }

    #[instrument(name = "okex_client.close_positions", skip(self), err)]
    pub async fn close_positions(&self, id: ClientOrderId) -> Result<(), OkexClientError> {
        let mut body: HashMap<String, String> = HashMap::new();
        body.insert(
            "instId".to_string(),
            OkexInstrumentId::BtcUsdSwap.to_string(),
        );
        body.insert("clOrdId".to_string(), id.0);
        body.insert("mgnMode".to_string(), OkexMarginMode::Cross.to_string());
        body.insert("posSide".to_string(), OkexPositionSide::Net.to_string());
        body.insert("ccy".to_string(), TradeCurrency::BTC.to_string());
        body.insert("autoCxl".to_string(), "false".to_string());
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

        match Self::extract_optional_response_data::<ClosePositionData>(response).await {
            Err(OkexClientError::UnexpectedResponse { msg, code })
                if code == "51023"
                    || msg.starts_with("Position does not exist")
                    || msg.starts_with("Position doesn't exist") =>
            {
                Ok(())
            }
            res => res.map(|_| ()),
        }
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
        Err(OkexClientError::from((msg, code)))
    }

    async fn extract_optional_response_data<T: serde::de::DeserializeOwned>(
        response: reqwest::Response,
    ) -> Result<Option<T>, OkexClientError> {
        let response_text = response.text().await?;
        let OkexResponse { code, msg, data } =
            serde_json::from_str::<OkexResponse<T>>(&response_text)?;
        if code == "0" && data.is_some() {
            if let Some(first) = data.unwrap().into_iter().next() {
                return Ok(Some(first));
            } else {
                return Ok(None);
            }
        }
        Err(OkexClientError::from((msg, code)))
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
        Err(OkexClientError::from((msg, code)))
    }

    fn sign_okex_request(&self, pre_hash: String) -> String {
        let key = hmac::Key::new(hmac::HMAC_SHA256, self.config.secret_key.as_bytes());
        let signature = hmac::sign(&key, pre_hash.as_bytes());
        BASE64.encode(signature.as_ref())
    }

    fn url_for_path(path: &str) -> String {
        format!("{OKEX_API_URL}{path}")
    }

    fn post_request_headers(
        &self,
        request_path: &str,
        request_body: &str,
    ) -> Result<HeaderMap, OkexClientError> {
        let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        let pre_hash = format!("{timestamp}POST{request_path}{request_body}");
        self.request_headers(timestamp, pre_hash)
    }

    fn get_request_headers(&self, request_path: &str) -> Result<HeaderMap, OkexClientError> {
        let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        let pre_hash = format!("{timestamp}GET{request_path}");
        self.request_headers(timestamp, pre_hash)
    }

    fn request_headers(
        &self,
        formatted_timestamp: String,
        pre_hash: String,
    ) -> Result<HeaderMap, OkexClientError> {
        let sign_base64 = self.sign_okex_request(pre_hash);
        let simulation_flag = i32::from(self.config.simulated);

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
            HeaderValue::from_str(simulation_flag.to_string().as_str())?,
        );

        Ok(headers)
    }
}
