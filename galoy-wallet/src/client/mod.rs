mod error;
mod queries;

use graphql_client::reqwest::post_graphql;
use reqwest::Client as ReqwestClient;

use error::*;
pub use queries::*;

use auth_code::*;
use btc_price::*;
use default_wallet::*;
use user_login::*;

#[derive(Debug, Clone)]
pub struct GaloyClient {
    client: ReqwestClient,
    config: GaloyClientConfig,
}

#[derive(Debug, Clone)]
pub struct GaloyClientConfig {
    pub api: String,
    pub phone_number: String,
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
        println!("{:?}", response);
        let response_data = response.data;
        println!("{:?}", response_data);

        if let Some(response_data) = response_data {
            let result = response_data.user_request_auth_code;

            return Ok(result);
        }
        Err(GaloyWalletError::UnknownResponse(
            "Failed to parse response data".to_string(),
        ))
    }

    pub async fn login(&self, auth_code: String) -> Result<UserLoginUserLogin, GaloyWalletError> {
        let variables = user_login::Variables {
            input: user_login::UserLoginInput {
                code: auth_code,
                phone: self.config.phone_number.clone(),
            },
        };

        let response =
            post_graphql::<UserLogin, _>(&self.client, &self.config.api, variables).await?;

        println!("{:#?}", response);
        let response_data = response.data;

        if let Some(response_data) = response_data {
            let result = response_data.user_login;

            return Ok(result);
        }
        Err(GaloyWalletError::UnknownResponse(
            "Failed to parse response data".to_string(),
        ))
    }

    pub async fn public_wallet(
        &self,
        username: String,
    ) -> Result<DefaultWalletAccountDefaultWallet, GaloyWalletError> {
        let input_variables = default_wallet::Variables {
            username,
            wallet_currency: Some(WalletCurrency::BTC),
        };
        let response =
            post_graphql::<DefaultWallet, _>(&self.client, &self.config.api, input_variables)
                .await?;

        println!("{:#?}", response);
        let response_data = response.data;

        if let Some(resp_data) = response_data {
            let result = resp_data.account_default_wallet;

            return Ok(result);
        }
        Err(GaloyWalletError::UnknownResponse(
            "Failed to parse response data".to_string(),
        ))
    }
}
