#![allow(clippy::enum_variant_names)]
#![allow(clippy::derive_partial_eq_without_eq)]

use chrono::{DateTime, Utc};
use graphql_client::GraphQLQuery;
use rust_decimal::Decimal;
use serde::Deserialize;

pub(super) type SafeInt = Decimal;

#[derive(Debug, PartialEq, Deserialize, Clone)]
pub struct GraphqlTimeStamp(#[serde(with = "chrono::serde::ts_seconds")] pub(super) DateTime<Utc>);

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
