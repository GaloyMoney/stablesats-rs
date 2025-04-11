mod config;
mod convert;
mod galoy_tracing;
mod queries;
mod transaction;

use galoy_tracing::*;
use graphql_client::{GraphQLQuery, Response};
use reqwest::{header::HeaderValue, Client as ReqwestClient, Method};
use tracing::instrument;

pub use self::convert::PathString;
use crate::error::*;
use queries::*;
pub use queries::{stablesats_transactions_list::WalletCurrency as SettlementCurrency, WalletId};

pub use config::*;
pub use transaction::*;

#[derive(Debug, Clone)]
pub struct GaloyClient {
    client: ReqwestClient,
    config: GaloyClientConfig,
}

impl GaloyClient {
    pub async fn connect(config: GaloyClientConfig) -> Result<Self, GaloyClientError> {
        if config.api_key.is_empty() {
            return Err(GaloyClientError::Authentication(
                "Empty API key".to_string(),
            ));
        }

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("X-API-KEY", HeaderValue::from_str(&config.api_key).unwrap());

        let client = ReqwestClient::builder()
            .use_rustls_tls()
            .default_headers(headers)
            .build()?;

        Ok(Self { client, config })
    }

    #[instrument(name = "galoy_client.transactions_list", skip(self), err)]
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
