mod error;
mod queries;

use futures::{
    stream::{self},
    Stream,
};
use graphql_client::{reqwest::post_graphql, GraphQLQuery, Response};
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    Client as ReqwestClient,
};

use error::*;
pub use queries::*;

use stablesats_auth_code::*;
use stablesats_transactions_list::*;
use stablesats_user_login::*;

// Type aliases
type StablesatsTransactions = StablesatsTransactionsListMeDefaultAccountWalletsTransactions;
type StablesatsTransactionsEdges =
    StablesatsTransactionsListMeDefaultAccountWalletsTransactionsEdges;
type StablesatsAuthenticationCode = StablesatsAuthCodeUserRequestAuthCode;

#[derive(Debug)]
pub struct StablesatsWalletsBalances {
    pub btc_wallet_balance: Option<queries::SignedAmount>,
    pub usd_wallet_balance: Option<queries::SignedAmount>,
}

#[derive(Debug)]
pub struct StablesatsWalletTransactions {
    pub btc_transactions: Option<StablesatsTransactions>,
    pub usd_transactions: Option<StablesatsTransactions>,
}

#[derive(Debug, Clone)]
pub struct GaloyClient {
    client: ReqwestClient,
    config: GaloyClientConfig,
    jwt: String,
}

#[derive(Debug, Clone)]
pub struct GaloyClientConfig {
    pub api: String,
    pub code: String,
    pub phone_number: String,
}

pub(crate) struct GaloyAuthToken {
    pub inner: Option<String>,
}

impl GaloyClient {
    pub async fn connect(config: GaloyClientConfig) -> Result<Self, GaloyClientError> {
        let jwt = Self::login_jwt(config.clone()).await?;
        let jwt = match jwt.inner {
            Some(jwt) => jwt,
            None => {
                return Err(GaloyClientError::AuthenticationToken(
                    "Empty authentication token".to_string(),
                ))
            }
        };

        Ok(Self {
            client: ReqwestClient::new(),
            config,
            jwt,
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
            let result = response_data.user_request_auth_code;

            return Ok(result);
        }
        Err(GaloyClientError::UnknownResponse(
            "Failed to parse response data".to_string(),
        ))
    }

    async fn login_jwt(config: GaloyClientConfig) -> Result<GaloyAuthToken, GaloyClientError> {
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
            return Err(GaloyClientError::UnknownResponse(format!(
                "Expected some response data, found {:?}",
                response_data
            )));
        }

        if let Some(response_data) = response_data {
            let user_login = response_data.user_login;
            let login_jwt = user_login.auth_token;

            return Ok(GaloyAuthToken { inner: login_jwt });
        }
        Err(GaloyClientError::UnknownResponse(
            "Failed to parse response data".to_string(),
        ))
    }

    pub async fn transactions_list(
        &mut self,
        last_transaction_cursor: String,
        wallet_currency: stablesats_transactions_list::WalletCurrency,
    ) -> Result<std::pin::Pin<Box<impl Stream<Item = StablesatsTransactionsEdges>>>, GaloyClientError>
    {
        let header_value = format!("Bearer {}", self.jwt);
        let mut header = HeaderMap::new();
        header.insert(AUTHORIZATION, HeaderValue::from_str(header_value.as_str())?);

        let variables = stablesats_transactions_list::Variables {
            last: None,
            first: None,
            before: None,
            after: Some(last_transaction_cursor),
        };

        let request_body = StablesatsTransactionsList::build_query(variables);
        let response = self
            .client
            .post(&self.config.api)
            .headers(header)
            .json(&request_body)
            .send()
            .await?;

        let response_body: Response<stablesats_transactions_list::ResponseData> =
            response.json().await?;
        let response_data = response_body.data;

        let result = match response_data {
            Some(data) => data,
            None => {
                return Err(GaloyClientError::UnknownResponse(
                    "Empty`data` response data".to_string(),
                ))
            }
        };

        let user = result.me;

        let default_account = match user {
            Some(data) => data.default_account,
            None => {
                return Err(GaloyClientError::UnknownResponse(
                    "Empty `me` response data".to_string(),
                ))
            }
        };

        let wallet = default_account
            .wallets
            .into_iter()
            .find(|wallet| wallet.wallet_currency == wallet_currency);

        if let Some(wallet) = wallet {
            let transaction_edges = match wallet.transactions {
                Some(tx) => tx.edges,
                None => {
                    return Err(GaloyClientError::UnknownResponse(
                        "Empty `transactions` response data".to_string(),
                    ))
                }
            };

            let transactions = match transaction_edges {
                Some(txs) => txs,
                None => {
                    return Err(GaloyClientError::UnknownResponse(
                        "Empty `edges` response data".to_string(),
                    ))
                }
            };

            return Ok(Box::pin(stream::iter(transactions)));
        }
        Err(GaloyClientError::UnknownResponse(
            "Failed to parse response data".to_string(),
        ))
    }

    pub async fn wallets_balances(&self) -> Result<StablesatsWalletsBalances, GaloyClientError> {
        let header_value = format!("Bearer {}", self.jwt);
        let mut header = HeaderMap::new();
        header.insert(AUTHORIZATION, HeaderValue::from_str(header_value.as_str())?);

        let variables = stablesats_wallets::Variables;
        let request_body = StablesatsWallets::build_query(variables);
        let response = self
            .client
            .post(&self.config.api)
            .headers(header)
            .json(&request_body)
            .send()
            .await?;

        let response_body: Response<stablesats_wallets::ResponseData> = response.json().await?;
        if response_body.errors.is_some() {
            if let Some(error) = response_body.errors {
                return Err(GaloyClientError::GrapqQlApi(error[0].clone().message));
            }
        }

        let response_data = response_body.data;
        if response_data.is_none() {
            return Err(GaloyClientError::UnknownResponse(
                "Empty `data` in response data".to_string(),
            ));
        }

        let me = match response_data {
            Some(data) => data.me,
            None => {
                return Err(GaloyClientError::UnknownResponse(
                    "Empty `data` in response data".to_string(),
                ))
            }
        };

        let default_account = match me {
            Some(me) => me.default_account,
            None => {
                return Err(GaloyClientError::UnknownResponse(
                    "Empty `me` in response data".to_string(),
                ))
            }
        };

        let wallets = default_account.wallets;

        let btc_wallet = wallets
            .clone()
            .into_iter()
            .find(|wallet| wallet.wallet_currency == stablesats_wallets::WalletCurrency::BTC);

        let usd_wallet = wallets
            .into_iter()
            .find(|wallet| wallet.wallet_currency == stablesats_wallets::WalletCurrency::USD);

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
}
