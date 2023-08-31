#![allow(clippy::enum_variant_names)]
#![allow(clippy::derive_partial_eq_without_eq)]

use chrono::{DateTime, Utc};
use graphql_client::GraphQLQuery;
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::GaloyClientError;

pub(super) type SafeInt = Decimal;

#[derive(Debug, PartialEq, Deserialize, Clone)]
pub struct GraphqlTimeStamp(#[serde(with = "chrono::serde::ts_seconds")] pub(super) DateTime<Utc>);

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/client/graphql/schema.graphql",
    query_path = "src/client/graphql/mutations/user_login.graphql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct StablesatsUserLogin;
pub type Phone = String;
pub type AuthToken = String;
pub type OneTimeAuthCode = String;

pub type StablesatsAuthToken = Option<String>;
impl TryFrom<stablesats_user_login::ResponseData> for StablesatsAuthToken {
    type Error = GaloyClientError;

    fn try_from(response: stablesats_user_login::ResponseData) -> Result<Self, Self::Error> {
        let user_login = response.user_login;
        let (auth_token, errors) = (user_login.auth_token, user_login.errors);

        if !errors.is_empty() {
            let error = errors[0].clone();

            return Err(GaloyClientError::GraphQLNested {
                message: error.message,
                path: error.path,
            });
        }
        Ok(auth_token)
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/client/graphql/schema.graphql",
    query_path = "src/client/graphql/queries/transactions_list.graphql",
    response_derives = "Debug, PartialEq, Clone"
)]
pub struct StablesatsTransactionsList;
pub type WalletId = String;

pub type Timestamp = GraphqlTimeStamp;
pub type Memo = String;
pub(crate) type SignedAmount = Decimal;
