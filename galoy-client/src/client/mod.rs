mod queries;

use graphql_client::reqwest::post_graphql;
use itertools::{Either, Itertools};
use reqwest::{
    header::{HeaderValue, AUTHORIZATION},
    Client as ReqwestClient,
};

pub use crate::error::*;
pub use queries::*;

use self::stablesats_on_chain_payment::PaymentSendResult;

pub type GaloyWalletCurrency = stablesats_transactions_list::WalletCurrency;

pub struct LastTransactionCursor(String);
impl From<String> for LastTransactionCursor {
    fn from(cursor: String) -> Self {
        Self(cursor)
    }
}

#[derive(Debug)]
pub struct StablesatsWalletsBalances {
    pub btc_wallet_balance: Option<queries::SignedAmount>,
    pub usd_wallet_balance: Option<queries::SignedAmount>,
}

#[derive(Debug, Clone)]
pub struct GaloyClient {
    client: ReqwestClient,
    config: GaloyClientConfig,
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

        Ok(Self { client, config })
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

    pub async fn transactions_list(
        &mut self,
        wallet_currency: GaloyWalletCurrency,
        last_transaction_cursor: Option<LastTransactionCursor>,
    ) -> Result<StablesatsTransactionsEdges, GaloyClientError> {
        let variables = stablesats_transactions_list::Variables {
            after: last_transaction_cursor.map(|cursor| cursor.0),
        };
        let response = post_graphql::<StablesatsTransactionsList, _>(
            &self.client,
            &self.config.api,
            variables,
        )
        .await?;
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
        let post_cursor_transactions = StablesatsTransactionsEdges::try_from(result)?;

        let tx_edges = post_cursor_transactions.edges;

        let (wanted_transactions, _): (Vec<_>, Vec<_>) =
            tx_edges.into_iter().partition_map(|tx_edge| {
                if tx_edge.node.settlement_currency == wallet_currency {
                    Either::Left(tx_edge)
                } else {
                    Either::Right(tx_edge)
                }
            });

        Ok(StablesatsTransactionsEdges {
            edges: wanted_transactions,
        })
    }

    pub async fn wallets_balances(&self) -> Result<StablesatsWalletsBalances, GaloyClientError> {
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

        let wallets = StablesatsWalletsWrapper::try_from(result)?;
        let btc_wallet = wallets.btc_wallet;
        let usd_wallet = wallets.usd_wallet;

        let btc_wallet_balance: Option<SignedAmount> = match btc_wallet {
            Some(wallet) => Some(wallet.balance),
            None => None,
        };
        let usd_wallet_balance: Option<SignedAmount> = match usd_wallet {
            Some(wallet) => Some(wallet.balance),
            None => None,
        };

        Ok(StablesatsWalletsBalances {
            usd_wallet_balance,
            btc_wallet_balance,
        })
    }

    pub async fn onchain_address(
        &self,
        wallet_id: WalletId,
    ) -> Result<OnchainAddress, GaloyClientError> {
        let variables = stablesats_deposit_address::Variables {
            input: stablesats_deposit_address::OnChainAddressCurrentInput { wallet_id },
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
        address: OnChainAddress,
        amount: SatAmount,
        memo: Option<Memo>,
        target_conf: TargetConfirmations,
        wallet_id: WalletId,
    ) -> Result<PaymentSendResult, GaloyClientError> {
        let variables = stablesats_on_chain_payment::Variables {
            input: stablesats_on_chain_payment::OnChainPaymentSendInput {
                address,
                amount,
                memo,
                target_confirmations: Some(target_conf),
                wallet_id,
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
        wallet_id: WalletId,
    ) -> Result<StablesatsTxFee, GaloyClientError> {
        let variables = stablesats_on_chain_tx_fee::Variables {
            address,
            amount,
            target_confirmations: Some(target_conf),
            wallet_id,
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
