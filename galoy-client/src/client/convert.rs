use super::{queries::*, WalletIds};
use crate::{error::*, stablesats_wallets::WalletCurrency};

impl TryFrom<stablesats_wallets::ResponseData> for WalletIds {
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
            if wallet.wallet_currency == WalletCurrency::BTC {
                btc = Some(wallet.id);
            } else {
                usd = Some(wallet.id);
            }
        }

        if let (Some(btc), Some(usd)) = (btc, usd) {
            Ok(Self { btc, usd })
        } else {
            Err(GaloyClientError::GrapqQlApi(
                "Missing `btc id` or `usd id` in response data".to_string(),
            ))
        }
    }
}
