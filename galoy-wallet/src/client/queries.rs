use graphql_client::GraphQLQuery;

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

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/client/graphql/schema.graphql",
    query_path = "src/client/graphql/mutations/user_login.graphql",
    response_derives = "Debug, PartialEq, Clone"
)]
pub struct UserLogin;
type AuthToken = String;
type OneTimeAuthCode = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/client/graphql/schema.graphql",
    query_path = "src/client/graphql/queries/transactions_list.graphql",
    response_derives = "Debug, PartialEq"
)]
pub struct TransactionsList;
type WalletId = String;
type Timestamp = u64;
type Memo = String;
type SignedAmount = f64;
