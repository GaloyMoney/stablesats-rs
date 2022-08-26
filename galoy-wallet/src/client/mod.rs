mod error;
mod queries;

use graphql_client::{reqwest::post_graphql, GraphQLQuery, Response};
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION},
    Client as ReqwestClient,
};

use error::*;
pub use queries::*;

use auth_code::*;
use btc_price::*;
use transactions_list::*;
use user_login::*;

pub struct GaloyDefaultWallets {
    pub btc_wallet: Option<wallets::WalletsMeDefaultAccountWallets>,
    pub usd_wallet: Option<wallets::WalletsMeDefaultAccountWallets>,
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
    pub jwt: String,
}

impl GaloyClient {
    pub fn new(config: GaloyClientConfig) -> Self {
        Self {
            client: ReqwestClient::new(),
            config,
        }
    }

    pub async fn btc_price(&self) -> Result<BtcPriceBtcPrice, GaloyWalletError> {
        let variables = btc_price::Variables;
        let response =
            post_graphql::<BtcPrice, _>(&self.client, &self.config.api, variables).await?;
        let response_data = response.data;

        if let Some(response_data) = response_data {
            let result = response_data.btc_price;
            if let Some(result) = result {
                return Ok(result);
            }
        }
        Err(GaloyWalletError::UnknownResponse(
            "Failed to parse response data".to_string(),
        ))
    }

    pub async fn authentication_code(
        &self,
    ) -> Result<AuthCodeUserRequestAuthCode, GaloyWalletError> {
        let phone_number = auth_code::Variables {
            input: auth_code::UserRequestAuthCodeInput {
                phone: self.config.phone_number.clone(),
            },
        };
        let response =
            post_graphql::<AuthCode, _>(&self.client, &self.config.api, phone_number).await?;
        let response_data = response.data;

        if let Some(response_data) = response_data {
            let result = response_data.user_request_auth_code;

            return Ok(result);
        }
        Err(GaloyWalletError::UnknownResponse(
            "Failed to parse response data".to_string(),
        ))
    }

    pub async fn login(&mut self) -> Result<UserLoginUserLogin, GaloyWalletError> {
        let variables = user_login::Variables {
            input: user_login::UserLoginInput {
                code: self.config.code.clone(),
                phone: self.config.phone_number.clone(),
            },
        };

        let response =
            post_graphql::<UserLogin, _>(&self.client, &self.config.api, variables).await?;

        let response_data = response.data;

        if let Some(response_data) = response_data {
            let result = response_data.user_login;

            // Update config JWT
            self.config.jwt = match result.clone().auth_token {
                Some(jwt) => jwt,
                None => "".to_string(),
            };

            return Ok(result);
        }
        Err(GaloyWalletError::UnknownResponse(
            "Failed to parse response data".to_string(),
        ))
    }

    pub async fn wallets(&self) -> Result<Option<GaloyDefaultWallets>, GaloyWalletError> {
        let header_value = format!("Bearer {}", self.config.jwt);
        let mut header = HeaderMap::new();
        header.insert(AUTHORIZATION, HeaderValue::from_str(header_value.as_str())?);

        let variables = wallets::Variables;
        let request_body = Wallets::build_query(variables);
        let response = self
            .client
            .post(&self.config.api)
            .headers(header)
            .json(&request_body)
            .send()
            .await?;

        let response_body: Response<wallets::ResponseData> = response.json().await?;
        let response_data = response_body.data;

        let me = match response_data {
            Some(data) => data.me,
            None => return Ok(None),
        };

        let default_wallet = match me {
            Some(me) => me.default_account,
            None => return Ok(None),
        };

        let wallets = default_wallet.wallets;

        let btc_wallet = wallets
            .clone()
            .into_iter()
            .find(|wallet| wallet.wallet_currency == wallets::WalletCurrency::BTC);

        let usd_wallet = wallets
            .into_iter()
            .find(|wallet| wallet.wallet_currency == wallets::WalletCurrency::USD);

        Ok(Some(GaloyDefaultWallets {
            btc_wallet,
            usd_wallet,
        }))
    }

    pub async fn transactions_list(
        &mut self,
        last_transaction_cursor: Option<String>,
    ) -> Result<Option<Vec<TransactionsListMeDefaultAccountTransactionsEdges>>, GaloyWalletError>
    {
        let header_value = format!("Bearer {}", self.config.jwt);
        let mut header = HeaderMap::new();
        header.insert(AUTHORIZATION, HeaderValue::from_str(header_value.as_str())?);

        let variables = transactions_list::Variables {
            last: None,
            first: None,
            before: None,
            after: last_transaction_cursor,
        };

        let request_body = TransactionsList::build_query(variables);
        let response = self
            .client
            .post(&self.config.api)
            .headers(header)
            .json(&request_body)
            .send()
            .await?;

        let response_body: Response<transactions_list::ResponseData> = response.json().await?;
        let response_data = response_body.data;

        let result = match response_data {
            Some(data) => data,
            None => return Ok(None),
        };

        let user = result.me;

        let default_account = match user {
            Some(data) => data.default_account,
            None => return Ok(None),
        };

        let transactions = default_account.transactions;
        let transactions_edges = match transactions {
            Some(data) => data.edges,
            None => return Ok(None),
        };

        Ok(transactions_edges)
    }

    pub async fn btc_transactions_list(
        &self,
        last_transaction_cursor: Option<String>,
    ) -> Result<(), GaloyWalletError> {
        // 1. Get BTC wallet id
        // 2. Use retrieved wallet id to retrieve transactions
        let header_value = format!("Bearer {}", self.config.jwt);
        let mut header = HeaderMap::new();
        header.insert(AUTHORIZATION, HeaderValue::from_str(header_value.as_str())?);

        let variables = transactions_list::Variables {
            last: None,
            first: None,
            before: None,
            after: last_transaction_cursor,
        };

        let request_body = TransactionsList::build_query(variables);
        let response = self
            .client
            .post(&self.config.api)
            .headers(header)
            .json(&request_body)
            .send()
            .await?;

        let response_body: Response<transactions_list::ResponseData> = response.json().await?;
        let response_data = response_body.data;
        println!("{:#?}", response_data);

        Ok(())
    }
}
