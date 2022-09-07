mod convert;
mod queries;

use graphql_client::reqwest::post_graphql;
use reqwest::{
    header::{HeaderValue, AUTHORIZATION},
    Client as ReqwestClient,
};
use rust_decimal::Decimal;

use crate::error::*;
use queries::*;
pub use queries::{
    stablesats_on_chain_payment::PaymentSendResult,
    stablesats_transactions_list::WalletCurrency as SettlementCurrency,
    stablesats_wallets::WalletCurrency, WalletId,
};

#[derive(Debug)]
pub struct LastTransactionCursor(String);
impl From<String> for LastTransactionCursor {
    fn from(cursor: String) -> Self {
        Self(cursor)
    }
}
pub type GaloyTransactionEdge =
    stablesats_transactions_list::StablesatsTransactionsListMeDefaultAccountTransactionsEdges;
#[derive(Debug)]
pub struct GaloyTransactions {
    pub cursor: Option<LastTransactionCursor>,
    pub list: Vec<GaloyTransactionEdge>,
    pub has_more: bool,
}

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

#[derive(Debug, Clone)]
pub struct GaloyClientConfig {
    pub api: String,
    pub code: String,
    pub phone_number: String,
}

pub(crate) struct StablesatsAuthToken {
    pub inner: Option<String>,
}

#[derive(Debug)]
pub struct OnchainAddress {
    pub address: String,
}

impl GaloyClient {
    pub async fn connect(config: GaloyClientConfig) -> Result<Self, GaloyClientError> {
        let jwt = Self::login_jwt(config.clone()).await?;
        let jwt = match jwt.inner {
            Some(jwt) => jwt,
            None => {
                return Err(GaloyClientError::Authentication(
                    "Empty authentication token".to_string(),
                ))
            }
        };
        let client = ReqwestClient::builder()
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

    pub async fn authentication_code(
        &self,
    ) -> Result<StablesatsAuthenticationCode, GaloyClientError> {
        let phone_number = stablesats_auth_code::Variables {
            input: stablesats_auth_code::UserRequestAuthCodeInput {
                phone: self.config.phone_number.clone(),
            },
        };
        let response =
            post_graphql::<StablesatsAuthCode, _>(&self.client, &self.config.api, phone_number)
                .await?;
        let response_data = response.data;

        if let Some(response_data) = response_data {
            StablesatsAuthenticationCode::try_from(response_data)?;
        }
        Err(GaloyClientError::GrapqQlApi(
            "Failed to parse response data".to_string(),
        ))
    }

    async fn login_jwt(config: GaloyClientConfig) -> Result<StablesatsAuthToken, GaloyClientError> {
        let variables = stablesats_user_login::Variables {
            input: stablesats_user_login::UserLoginInput {
                code: config.code.clone(),
                phone: config.phone_number.clone(),
            },
        };

        let client = ReqwestClient::new();

        let response =
            post_graphql::<StablesatsUserLogin, _>(&client, config.api, variables).await?;
        if response.errors.is_some() {
            if let Some(error) = response.errors {
                return Err(GaloyClientError::GrapqQlApi(error[0].clone().message));
            }
        }

        let response_data = response.data;
        if response_data.is_none() {
            return Err(GaloyClientError::GrapqQlApi(
                "Empty `data` in response".to_string(),
            ));
        }

        let auth_token = match response_data {
            Some(login_data) => StablesatsLogin::try_from(login_data)?.auth_token,
            None => {
                return Err(GaloyClientError::GrapqQlApi(format!(
                    "Expected some response data, found {:?}",
                    response_data
                )))
            }
        };
        Ok(StablesatsAuthToken { inner: auth_token })
    }

    async fn wallet_ids(
        client: ReqwestClient,
        config: GaloyClientConfig,
    ) -> Result<WalletId, GaloyClientError> {
        let variables = stablesats_wallets::Variables;
        let response =
            post_graphql::<StablesatsWallets, _>(&client, &config.api, variables).await?;
        if response.errors.is_some() {
            if let Some(error) = response.errors {
                return Err(GaloyClientError::GrapqQlApi(error[0].clone().message));
            }
        }

        let response_data = response.data;
        let result = match response_data {
            Some(result) => result,
            None => {
                return Err(GaloyClientError::GrapqQlApi(
                    "Empty `me` in response data".to_string(),
                ))
            }
        };

        let wallet_id = WalletId::try_from(result)?;

        Ok(wallet_id)
    }

    pub async fn transactions_list(
        &mut self,
        cursor: Option<LastTransactionCursor>,
    ) -> Result<GaloyTransactions, GaloyClientError> {
        let variables = stablesats_transactions_list::Variables {
            last: None,
            before: cursor.map(|cursor| cursor.0),
        };
        let response = post_graphql::<StablesatsTransactionsList, _>(
            &self.client,
            &self.config.api,
            variables,
        )
        .await?;
        if let Some(error) = response.errors {
            return Err(GaloyClientError::GrapqQlApi(error[0].clone().message));
        }

        let result = response.data.ok_or_else(|| {
            GaloyClientError::GrapqQlApi("Empty `me` in response data".to_string())
        })?;
        GaloyTransactions::try_from(result)
    }

    pub async fn wallet_balances(&self) -> Result<WalletBalances, GaloyClientError> {
        let variables = stablesats_wallets::Variables;
        let response =
            post_graphql::<StablesatsWallets, _>(&self.client, &self.config.api, variables).await?;
        if response.errors.is_some() {
            if let Some(error) = response.errors {
                return Err(GaloyClientError::GrapqQlApi(error[0].clone().message));
            }
        }

        let response_data = response.data;
        let result = match response_data {
            Some(result) => result,
            None => {
                return Err(GaloyClientError::GrapqQlApi(
                    "Empty `me` in response data".to_string(),
                ))
            }
        };

        WalletBalances::try_from(result)
    }

    pub async fn onchain_address(&self) -> Result<OnchainAddress, GaloyClientError> {
        let variables = stablesats_deposit_address::Variables {
            input: stablesats_deposit_address::OnChainAddressCurrentInput {
                wallet_id: self.btc_wallet_id.clone(),
            },
        };
        let response =
            post_graphql::<StablesatsDepositAddress, _>(&self.client, &self.config.api, variables)
                .await?;
        if response.errors.is_some() {
            if let Some(error) = response.errors {
                return Err(GaloyClientError::GrapqQlApi(error[0].clone().message));
            }
        }

        let response_data = response.data;
        let result = match response_data {
            Some(data) => data,
            None => {
                return Err(GaloyClientError::GrapqQlApi(
                    "Empty `on chain address create` in response data".to_string(),
                ))
            }
        };

        let onchain_address_create = StablesatsOnchainAddress::try_from(result)?;
        let address = match onchain_address_create.address {
            Some(address) => address,
            None => {
                return Err(GaloyClientError::GrapqQlApi(
                    "Empty `address` in response data".to_string(),
                ))
            }
        };

        Ok(OnchainAddress { address })
    }

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
        let response =
            post_graphql::<StablesatsOnChainPayment, _>(&self.client, &self.config.api, variables)
                .await?;
        if response.errors.is_some() {
            if let Some(error) = response.errors {
                return Err(GaloyClientError::GrapqQlApi(error[0].clone().message));
            }
        }

        let response_data = response.data;
        let result = match response_data {
            Some(data) => data,
            None => {
                return Err(GaloyClientError::GrapqQlApi(
                    "Empty `onChainPaymentSend` in response data".to_string(),
                ))
            }
        };

        let onchain_payment_send = StablesatsPaymentSend::try_from(result)?;
        if !onchain_payment_send.errors.is_empty() {
            return Err(GaloyClientError::GrapqQlApi(
                onchain_payment_send.errors[0].clone().message,
            ));
        };

        let payment_status = onchain_payment_send.status;
        let status = match payment_status {
            Some(status) => status,
            None => {
                return Err(GaloyClientError::GrapqQlApi(
                    "Empty `status` in response data".to_string(),
                ))
            }
        };

        Ok(status)
    }

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
        let response =
            post_graphql::<StablesatsOnChainTxFee, _>(&self.client, &self.config.api, variables)
                .await?;
        if response.errors.is_some() {
            if let Some(error) = response.errors {
                return Err(GaloyClientError::GrapqQlApi(error[0].clone().message));
            }
        }

        let response_data = response.data;
        let result = match response_data {
            Some(data) => data,
            None => {
                return Err(GaloyClientError::GrapqQlApi(
                    "Empty `onChainTxFee` in response data".to_string(),
                ))
            }
        };

        let onchain_tx_fee = StablesatsTxFee::try_from(result)?;

        Ok(onchain_tx_fee)
    }
}
