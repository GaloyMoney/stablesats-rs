mod config;
mod convert;
mod galoy_tracing;
mod queries;
mod transaction;

use galoy_tracing::*;
use graphql_client::{GraphQLQuery, Response};
use reqwest::{
    header::{HeaderValue, AUTHORIZATION},
    Client as ReqwestClient, Method,
};
use rust_decimal::Decimal;
use tracing::instrument;

use crate::error::*;
use queries::*;
pub use queries::{
    stablesats_on_chain_payment::PaymentSendResult,
    stablesats_transactions_list::WalletCurrency as SettlementCurrency,
    stablesats_wallets::WalletCurrency, StablesatsAuthToken, WalletId,
};

pub use config::*;
pub use transaction::*;

pub use self::convert::PathString;

#[derive(Debug)]
pub struct WalletBalances {
    pub btc: Decimal,
    pub usd: Decimal,
}

#[derive(Debug, Clone)]
pub struct GaloyClient {
    client: ReqwestClient,
    config: GaloyClientConfig,
    btc_wallet_id: String,
}

#[derive(Debug)]
pub struct OnchainAddress {
    pub address: String,
}

impl GaloyClient {
    pub async fn connect(config: GaloyClientConfig) -> Result<Self, GaloyClientError> {
        let jwt = Self::login_jwt(config.clone()).await?;
        let jwt = jwt.ok_or_else(|| {
            GaloyClientError::Authentication("Empty authentication token".to_string())
        })?;
        let client = ReqwestClient::builder()
            .use_rustls_tls()
            .default_headers(
                std::iter::once((
                    AUTHORIZATION,
                    HeaderValue::from_str(&format!("Bearer {}", jwt)).unwrap(),
                ))
                .collect(),
            )
            .build()?;

        let btc_wallet_id = Self::wallet_ids(client.clone(), config.clone()).await?;

        Ok(Self {
            client,
            config,
            btc_wallet_id,
        })
    }

    #[instrument(name = "galoy_authentication_code", err)]
    pub async fn authentication_code(
        &self,
    ) -> Result<StablesatsAuthenticationCode, GaloyClientError> {
        let phone_number = stablesats_auth_code::Variables {
            input: stablesats_auth_code::UserRequestAuthCodeInput {
                phone: self.config.phone_number.clone(),
            },
        };
        let response = GaloyClient::traced_gql_request::<StablesatsAuthCode, _>(
            &self.client,
            &self.config.api,
            phone_number,
        )
        .await?;
        let response_data = response
            .data
            .ok_or_else(|| GaloyClientError::GraphQLNested {
                message: "Empty authentication code response data".to_string(),
                path: None,
            })?;

        let auth_code = StablesatsAuthenticationCode::try_from(response_data)?;

        Ok(auth_code)
    }

    #[instrument(name = "galoy_login_jwt", skip_all, err)]
    async fn login_jwt(config: GaloyClientConfig) -> Result<StablesatsAuthToken, GaloyClientError> {
        let variables = stablesats_user_login::Variables {
            input: stablesats_user_login::UserLoginInput {
                code: config.auth_code.clone(),
                phone: config.phone_number.clone(),
            },
        };

        let client = ReqwestClient::builder().use_rustls_tls().build()?;

        let response = GaloyClient::traced_gql_request::<StablesatsUserLogin, _>(
            &client, config.api, variables,
        )
        .await?;

        if response.errors.is_some() {
            if let Some(errors) = response.errors {
                let zeroth_error = errors[0].clone();

                return Err(GaloyClientError::GraphQLTopLevel {
                    message: zeroth_error.message,
                    path: zeroth_error.path.into(),
                    locations: zeroth_error.locations,
                    extensions: zeroth_error.extensions,
                });
            }
        }

        let response_data = response
            .data
            .ok_or_else(|| GaloyClientError::GraphQLNested {
                message: "Empty `data` in response".to_string(),
                path: None,
            })?;

        let auth_token = StablesatsAuthToken::try_from(response_data)?;

        Ok(auth_token)
    }

    #[instrument(name = "galoy_wallet_ids", skip_all, err)]
    async fn wallet_ids(
        client: ReqwestClient,
        config: GaloyClientConfig,
    ) -> Result<WalletId, GaloyClientError> {
        let variables = stablesats_wallets::Variables;
        let response = GaloyClient::traced_gql_request::<StablesatsWallets, _>(
            &client,
            &config.api,
            variables,
        )
        .await?;
        if response.errors.is_some() {
            if let Some(errors) = response.errors {
                let zeroth_error = errors[0].clone();

                return Err(GaloyClientError::GraphQLTopLevel {
                    message: zeroth_error.message,
                    path: zeroth_error.path.into(),
                    locations: zeroth_error.locations,
                    extensions: zeroth_error.extensions,
                });
            }
        }

        let response_data = response.data;
        let result = response_data.ok_or_else(|| GaloyClientError::GraphQLNested {
            message: "Empty `me` in response data".to_string(),
            path: None,
        })?;

        let wallet_id = WalletId::try_from(result)?;

        Ok(wallet_id)
    }

    #[instrument(name = "galoy_transactions_list", skip(self), err)]
    pub async fn transactions_list(
        &self,
        cursor: Option<TxCursor>,
    ) -> Result<GaloyTransactions, GaloyClientError> {
        let variables = stablesats_transactions_list::Variables {
            last: Some(100),
            before: cursor.map(|cursor| cursor.0),
        };

        let response = GaloyClient::traced_gql_request::<StablesatsTransactionsList, _>(
            &self.client,
            &self.config.api,
            variables,
        )
        .await?;
        if let Some(errors) = response.errors {
            let zeroth_error = errors[0].clone();

            return Err(GaloyClientError::GraphQLTopLevel {
                message: zeroth_error.message,
                path: zeroth_error.path.into(),
                locations: zeroth_error.locations,
                extensions: zeroth_error.extensions,
            });
        }

        let result = response
            .data
            .ok_or_else(|| GaloyClientError::GraphQLNested {
                message: "Empty `me` in response data".to_string(),
                path: None,
            })?;
        GaloyTransactions::try_from(result)
    }

    #[instrument(name = "galoy_wallet_balances", skip(self), err)]
    pub async fn wallet_balances(&self) -> Result<WalletBalances, GaloyClientError> {
        let variables = stablesats_wallets::Variables;
        let response = GaloyClient::traced_gql_request::<StablesatsWallets, _>(
            &self.client,
            &self.config.api,
            variables,
        )
        .await?;
        if response.errors.is_some() {
            if let Some(errors) = response.errors {
                let zeroth_error = errors[0].clone();

                return Err(GaloyClientError::GraphQLTopLevel {
                    message: zeroth_error.message,
                    path: zeroth_error.path.into(),
                    locations: zeroth_error.locations,
                    extensions: zeroth_error.extensions,
                });
            }
        }

        let response_data = response.data;
        let result = response_data.ok_or_else(|| GaloyClientError::GraphQLNested {
            message: "Empty `me` in response data".to_string(),
            path: None,
        })?;

        WalletBalances::try_from(result)
    }

    #[instrument(name = "galoy_onchain_address", skip(self), err)]
    pub async fn onchain_address(&self) -> Result<OnchainAddress, GaloyClientError> {
        let variables = stablesats_deposit_address::Variables {
            input: stablesats_deposit_address::OnChainAddressCurrentInput {
                wallet_id: self.btc_wallet_id.clone(),
            },
        };
        let response = GaloyClient::traced_gql_request::<StablesatsDepositAddress, _>(
            &self.client,
            &self.config.api,
            variables,
        )
        .await?;
        if response.errors.is_some() {
            if let Some(errors) = response.errors {
                let zeroth_error = errors[0].clone();

                return Err(GaloyClientError::GraphQLTopLevel {
                    message: zeroth_error.message,
                    path: zeroth_error.path.into(),
                    locations: zeroth_error.locations,
                    extensions: zeroth_error.extensions,
                });
            }
        }

        let response_data = response.data;
        let result = response_data.ok_or_else(|| GaloyClientError::GraphQLNested {
            message: "Empty `on chain address create` in response data".to_string(),
            path: None,
        })?;

        let onchain_address_create = StablesatsOnchainAddress::try_from(result)?;
        let address =
            onchain_address_create
                .address
                .ok_or_else(|| GaloyClientError::GraphQLNested {
                    message: "Empty `address` in response data".to_string(),
                    path: None,
                })?;

        Ok(OnchainAddress { address })
    }

    #[instrument(name = "galoy_send_onchain_payment", skip(self), err)]
    pub async fn send_onchain_payment(
        &self,
        address: String,
        amount: Decimal,
        memo: Option<Memo>,
        target_conf: TargetConfirmations,
    ) -> Result<PaymentSendResult, GaloyClientError> {
        let variables = stablesats_on_chain_payment::Variables {
            input: stablesats_on_chain_payment::OnChainPaymentSendInput {
                address,
                amount,
                memo,
                target_confirmations: Some(target_conf),
                wallet_id: self.btc_wallet_id.clone(),
            },
        };
        let response = GaloyClient::traced_gql_request::<StablesatsOnChainPayment, _>(
            &self.client,
            &self.config.api,
            variables,
        )
        .await?;
        if response.errors.is_some() {
            if let Some(errors) = response.errors {
                let zeroth_error = errors[0].clone();

                return Err(GaloyClientError::GraphQLTopLevel {
                    message: zeroth_error.message,
                    path: zeroth_error.path.into(),
                    locations: zeroth_error.locations,
                    extensions: zeroth_error.extensions,
                });
            }
        }

        let response_data = response.data;
        let result = response_data.ok_or_else(|| GaloyClientError::GraphQLNested {
            message: "Empty `onChainPaymentSend` in response data".to_string(),
            path: None,
        })?;

        let onchain_payment_send = StablesatsPaymentSend::try_from(result)?;
        if !onchain_payment_send.errors.is_empty() {
            let zeroth_error = onchain_payment_send.errors[0].clone();
            return Err(GaloyClientError::GraphQLNested {
                message: zeroth_error.message,
                path: zeroth_error.path,
            });
        };

        let payment_status = onchain_payment_send.status;
        let status = payment_status.ok_or_else(|| GaloyClientError::GraphQLNested {
            message: "Empty `status` in response data".to_string(),
            path: None,
        })?;

        Ok(status)
    }

    #[instrument(name = "galoy_onchain_tx_fee", skip(self), err)]
    pub async fn onchain_tx_fee(
        &self,
        address: OnChainAddress,
        amount: SatAmount,
        target_conf: TargetConfirmations,
    ) -> Result<StablesatsTxFee, GaloyClientError> {
        let variables = stablesats_on_chain_tx_fee::Variables {
            address,
            amount,
            target_confirmations: Some(target_conf),
            wallet_id: self.btc_wallet_id.clone(),
        };
        let response = GaloyClient::traced_gql_request::<StablesatsOnChainTxFee, _>(
            &self.client,
            &self.config.api,
            variables,
        )
        .await?;
        if response.errors.is_some() {
            if let Some(errors) = response.errors {
                let zeroth_error = errors[0].clone();

                return Err(GaloyClientError::GraphQLTopLevel {
                    message: zeroth_error.message,
                    path: zeroth_error.path.into(),
                    locations: zeroth_error.locations,
                    extensions: zeroth_error.extensions,
                });
            }
        }

        let response_data = response.data;
        let result = response_data.ok_or_else(|| GaloyClientError::GraphQLNested {
            message: "Empty `onChainTxFee` in response data".to_string(),
            path: None,
        })?;

        let onchain_tx_fee = StablesatsTxFee::try_from(result)?;

        Ok(onchain_tx_fee)
    }

    async fn traced_gql_request<Q: GraphQLQuery, U: reqwest::IntoUrl>(
        client: &ReqwestClient,
        url: U,
        variables: Q::Variables,
    ) -> Result<Response<Q::ResponseData>, GaloyClientError> {
        let trace_headers = inject_trace();
        let body = Q::build_query(variables);
        let response = client
            .request(Method::POST, url)
            .headers(trace_headers)
            .json(&body)
            .send()
            .await?;

        let response = response.json::<Response<Q::ResponseData>>().await?;

        Ok(response)
    }
}
