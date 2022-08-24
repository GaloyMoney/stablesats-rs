mod error;

use error::*;
use graphql_client::{reqwest::post_graphql, GraphQLQuery};
use reqwest::Client as ReqwestClient;

use auth_code::*;
use btc_price::*;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/client/graphql/schema.graphql",
    query_path = "src/client/graphql/queries/btc_price.graphql",
    response_derives = "Debug, PartialEq"
)]
pub struct BtcPrice;
type SafeInt = i64;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/client/graphql/schema.graphql",
    query_path = "src/client/graphql/mutations/user_request_auth_code.graphql",
    response_derives = "Debug, PartialEq"
)]
pub struct AuthCode;
type Phone = String;

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
            post_graphql::<AuthCode, _>(&self.client, &self.config.phone_number, phone_number)
                .await?;
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
}
