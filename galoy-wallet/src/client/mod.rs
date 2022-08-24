mod error;

use error::*;
use graphql_client::{reqwest::post_graphql, GraphQLQuery};
use reqwest::Client as ReqwestClient;

use btc_price::*;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/client/graphql/schema.graphql",
    query_path = "src/client/graphql/query_btc_price.graphql",
    response_derives = "Debug, PartialEq"
)]
pub struct BtcPrice;
type SafeInt = i64;

pub struct GaloyClient {
    client: ReqwestClient,

    api: String,
}

impl GaloyClient {
    pub fn new(api: String) -> Self {
        Self {
            client: ReqwestClient::new(),
            api,
        }
    }

    pub async fn btc_price(&self) -> Result<BtcPriceBtcPrice, GaloyWalletError> {
        let variables = btc_price::Variables;
        let response = post_graphql::<BtcPrice, _>(&self.client, &self.api, variables).await?;
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
}
