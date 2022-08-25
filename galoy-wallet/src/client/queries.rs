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
    response_derives = "Debug, PartialEq"
)]
pub struct UserLogin;
type AuthToken = String;
type OneTimeAuthCode = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/client/graphql/schema.graphql",
    query_path = "src/client/graphql/queries/default_wallet.graphql",
    response_derives = "Debug, PartialEq"
)]
pub struct DefaultWallet;
type Username = String;
