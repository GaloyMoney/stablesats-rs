use graphql_client::GraphQLQuery;
use rust_decimal::Decimal;

use crate::GaloyClientError;

use self::stablesats_wallets::{StablesatsWalletsMeDefaultAccountWallets, WalletCurrency};

pub type SafeInt = i64;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/client/graphql/schema.graphql",
    query_path = "src/client/graphql/mutations/user_request_auth_code.graphql",
    response_derives = "Debug, PartialEq, Eq"
)]
pub struct StablesatsAuthCode;
pub type Phone = String;
pub type StablesatsAuthenticationCode = stablesats_auth_code::StablesatsAuthCodeUserRequestAuthCode;
impl TryFrom<stablesats_auth_code::ResponseData> for StablesatsAuthenticationCode {
    type Error = GaloyClientError;

    fn try_from(response: stablesats_auth_code::ResponseData) -> Result<Self, Self::Error> {
        let auth_code = response.user_request_auth_code;
        if let Some(is_success) = auth_code.success {
            if !is_success {
                return Err(GaloyClientError::Authentication(
                    "Authentication failed".to_string(),
                ));
            } else {
                return Ok(auth_code);
            }
        }

        Err(GaloyClientError::Authentication(
            "Empty authentication code".to_string(),
        ))
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/client/graphql/schema.graphql",
    query_path = "src/client/graphql/mutations/user_login.graphql",
    response_derives = "Debug, PartialEq, Eq, Clone"
)]
pub struct StablesatsUserLogin;
pub type AuthToken = String;
pub type OneTimeAuthCode = String;
pub type StablesatsLogin = stablesats_user_login::StablesatsUserLoginUserLogin;
impl TryFrom<stablesats_user_login::ResponseData> for StablesatsLogin {
    type Error = GaloyClientError;

    fn try_from(response: stablesats_user_login::ResponseData) -> Result<Self, Self::Error> {
        let user_login = response.user_login;
        Ok(user_login)
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
pub type Timestamp = u64; // change to chrono
pub type Memo = String;
pub(crate) type SignedAmount = Decimal;

pub type StablesatsTransactions =
    stablesats_transactions_list::StablesatsTransactionsListMeDefaultAccountTransactionsEdges;

#[derive(Debug)]
pub struct StablesatsTransactionsEdges {
    pub edges: Vec<StablesatsTransactions>,
}

impl TryFrom<stablesats_transactions_list::ResponseData> for StablesatsTransactionsEdges {
    type Error = GaloyClientError;

    fn try_from(response: stablesats_transactions_list::ResponseData) -> Result<Self, Self::Error> {
        let me = response.me;
        let me = match me {
            Some(me) => me,
            None => {
                return Err(GaloyClientError::GrapqQlApi(
                    "Empty `me` in response data".to_string(),
                ))
            }
        };

        let default_account = me.default_account;
        let transactions = default_account.transactions;

        let transactions = match transactions {
            Some(tx) => tx,
            None => {
                return Err(GaloyClientError::GrapqQlApi(
                    "Empty `transactions` in response data".to_string(),
                ))
            }
        };

        let edges = match transactions.edges {
            Some(edges) => edges,
            None => {
                return Err(GaloyClientError::GrapqQlApi(
                    "Empty `transaction edges` in response data".to_string(),
                ))
            }
        };
        Ok(StablesatsTransactionsEdges { edges })
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/client/graphql/schema.graphql",
    query_path = "src/client/graphql/queries/wallets.graphql",
    response_derives = "Debug, PartialEq, Clone"
)]
pub struct StablesatsWallets;
pub type StablesatsWalletsWrap = StablesatsWalletsWrapper;

pub struct StablesatsWalletsWrapper {
    pub btc_wallet: Option<StablesatsWalletsMeDefaultAccountWallets>,
    pub usd_wallet: Option<StablesatsWalletsMeDefaultAccountWallets>,
}

impl TryFrom<stablesats_wallets::ResponseData> for StablesatsWalletsWrapper {
    type Error = GaloyClientError;

    fn try_from(response: stablesats_wallets::ResponseData) -> Result<Self, Self::Error> {
        let me = response.me;
        let me = match me {
            Some(me) => me,
            None => {
                return Err(GaloyClientError::GrapqQlApi(
                    "Empty `me` in response data".to_string(),
                ))
            }
        };
        let default_account = me.default_account;
        let wallets = default_account.wallets;

        let btc_wallet = wallets
            .clone()
            .into_iter()
            .find(|wallet| wallet.wallet_currency == WalletCurrency::BTC);

        let usd_wallet = wallets
            .into_iter()
            .find(|wallet| wallet.wallet_currency == WalletCurrency::USD);

        Ok(StablesatsWalletsWrapper {
            btc_wallet,
            usd_wallet,
        })
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/client/graphql/schema.graphql",
    query_path = "src/client/graphql/queries/onchain_tx_fee.graphql",
    response_derives = "Debug"
)]
pub struct StablesatsOnChainTxFee;
pub type TargetConfirmations = u32;
pub type SatAmount = Decimal;
pub type OnChainAddress = String;

pub type StablesatsTxFee = stablesats_on_chain_tx_fee::StablesatsOnChainTxFeeOnChainTxFee;

impl TryFrom<stablesats_on_chain_tx_fee::ResponseData> for StablesatsTxFee {
    type Error = GaloyClientError;

    fn try_from(response: stablesats_on_chain_tx_fee::ResponseData) -> Result<Self, Self::Error> {
        let onchain_tx_fee = response.on_chain_tx_fee;
        Ok(onchain_tx_fee)
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/client/graphql/schema.graphql",
    query_path = "src/client/graphql/mutations/deposit_address.graphql",
    response_derives = "Debug"
)]
pub struct StablesatsDepositAddress;

pub type StablesatsOnchainAddress =
    stablesats_deposit_address::StablesatsDepositAddressOnChainAddressCurrent;

impl TryFrom<stablesats_deposit_address::ResponseData> for StablesatsOnchainAddress {
    type Error = GaloyClientError;

    fn try_from(response: stablesats_deposit_address::ResponseData) -> Result<Self, Self::Error> {
        let create_address = response.on_chain_address_current;
        Ok(create_address)
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/client/graphql/schema.graphql",
    query_path = "src/client/graphql/mutations/onchain_payment.graphql",
    response_derives = "Debug, Clone, PartialEq"
)]
pub struct StablesatsOnChainPayment;

pub type StablesatsPaymentSend =
    stablesats_on_chain_payment::StablesatsOnChainPaymentOnChainPaymentSend;

impl TryFrom<stablesats_on_chain_payment::ResponseData> for StablesatsPaymentSend {
    type Error = GaloyClientError;

    fn try_from(response: stablesats_on_chain_payment::ResponseData) -> Result<Self, Self::Error> {
        let onchain_payment_send = response.on_chain_payment_send;
        Ok(onchain_payment_send)
    }
}
