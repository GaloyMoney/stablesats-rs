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
use tracing::instrument;

pub use self::convert::PathString;
use crate::error::*;
use queries::*;
pub use queries::{
    stablesats_transactions_list::WalletCurrency as SettlementCurrency, StablesatsAuthToken,
    WalletId,
};

pub use config::*;
pub use transaction::*;

#[derive(Debug, Clone)]
pub struct GaloyClient {
    client: ReqwestClient,
    config: GaloyClientConfig,
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
                    HeaderValue::from_str(&format!("Bearer {jwt}")).unwrap(),
                ))
                .collect(),
            )
            .build()?;

        Ok(Self { client, config })
    }

    #[instrument(name = "galoy_client.login_jwt", skip_all, err)]
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
