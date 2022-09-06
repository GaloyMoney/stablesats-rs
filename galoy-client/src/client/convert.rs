use super::{queries::*, GaloyTransactions, LastTransactionCursor, WalletBalances, WalletCurrency};
use crate::error::*;

impl TryFrom<stablesats_wallets::ResponseData> for WalletBalances {
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

        let mut btc = None;
        let mut usd = None;

        for wallet in wallets {
            let currency = wallet.wallet_currency;
            let balance = wallet.balance;

            match currency {
                stablesats_wallets::WalletCurrency::BTC => {
                    btc = Some(balance);
                }
                stablesats_wallets::WalletCurrency::USD => {
                    usd = Some(balance);
                }
                _ => {}
            }
        }

        if let (Some(btc), Some(usd)) = (btc, usd) {
            Ok(Self { btc, usd })
        } else {
            Err(GaloyClientError::GrapqQlApi(
                "Missing `btc` or `usd` in response data".to_string(),
            ))
        }
    }
}

impl TryFrom<stablesats_wallets::ResponseData> for WalletId {
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

        let mut btc_id = None;

        for wallet in wallets {
            if wallet.wallet_currency == WalletCurrency::BTC {
                btc_id = Some(wallet.id);
            }
        }

        if let Some(btc_id) = btc_id {
            Ok(btc_id)
        } else {
            Err(GaloyClientError::GrapqQlApi(
                "Missing `btc id` in response data".to_string(),
            ))
        }
    }
}

impl TryFrom<stablesats_transactions_list::ResponseData> for GaloyTransactions {
    type Error = GaloyClientError;

    fn try_from(response: stablesats_transactions_list::ResponseData) -> Result<Self, Self::Error> {
        let me = response.me.ok_or_else(|| {
            GaloyClientError::GrapqQlApi("Empty `me` in response data".to_string())
        })?;

        let transactions = me.default_account.transactions.ok_or_else(|| {
            GaloyClientError::GrapqQlApi("Empty `transactions` in response data".to_string())
        })?;
        let page_info = transactions.page_info;
        let edges = transactions.edges.ok_or_else(|| {
            GaloyClientError::GrapqQlApi("Empty `transaction edges` in response data".to_string())
        })?;
        let list = edges.into_iter().map(|edge| edge.node).collect::<Vec<_>>();

        Ok(Self {
            list,
            cursor: page_info.end_cursor.map(LastTransactionCursor::from),
            has_more: page_info.has_next_page,
        })
    }
}
