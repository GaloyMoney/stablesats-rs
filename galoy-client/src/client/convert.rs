use super::{queries::*, WalletBalances};
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
