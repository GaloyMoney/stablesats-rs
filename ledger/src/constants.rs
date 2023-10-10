use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::{uuid, Uuid};

// Templates
pub(super) const USER_BUYS_USD_CODE: &str = "USER_BUYS_USD";
pub(super) const USER_BUYS_USD_ID: Uuid = uuid!("00000000-0000-0000-0000-000000000001");
pub(super) const REVERT_USER_BUYS_USD_CODE: &str = "REVERT_USER_BUYS_USD";
pub(super) const REVERT_USER_BUYS_USD_ID: Uuid = uuid!("00000000-0000-0000-0000-100000000001");
pub(super) const USER_SELLS_USD_CODE: &str = "USER_SELLS_USD";
pub(super) const USER_SELLS_USD_ID: Uuid = uuid!("00000000-0000-0000-0000-000000000002");
pub(super) const REVERT_USER_SELLS_USD_CODE: &str = "REVERT_USER_SELLS_USD";
pub(super) const REVERT_USER_SELLS_USD_ID: Uuid = uuid!("00000000-0000-0000-0000-100000000002");
pub(super) const INCREASE_EXCHANGE_POSITION_CODE: &str = "INCREASE_EXCHANGE_POSITION";
pub(super) const INCREASE_EXCHANGE_POSITION_ID: Uuid =
    uuid!("00000000-0000-0000-0000-000000000003");
pub(super) const DECREASE_EXCHANGE_POSITION_CODE: &str = "DECREASE_EXCHANGE_POSITION";
pub(super) const DECREASE_EXCHANGE_POSITION_ID: Uuid =
    uuid!("00000000-0000-0000-0000-000000000004");
pub(super) const QUOTE_BUY_USD_CODE: &str = "QUOTE_BUY_USD";
pub(super) const QUOTE_BUY_USD_ID: Uuid = uuid!("00000000-0000-0000-0000-000000000005");
pub(super) const REVERT_QUOTE_BUY_USD_CODE: &str = "REVERT_QUOTE_BUY_USD";
pub(super) const REVERT_QUOTE_BUY_USD_ID: Uuid = uuid!("00000000-0000-0000-0000-100000000005");

// Journal
pub(super) const STABLESATS_JOURNAL_NAME: &str = "Stablesats";
pub(super) const STABLESATS_JOURNAL_ID: Uuid = uuid!("00000000-0000-0000-0000-000000000001");

pub(super) const EXCHANGE_POSITION_JOURNAL_NAME: &str = "Exchange Position";
pub(super) const EXCHANGE_POSITION_JOURNAL_ID: Uuid = uuid!("00000000-0000-0000-0000-000000000002");

// Accounts
pub(super) const EXTERNAL_OMNIBUS_CODE: &str = "EXTERNAL_OMNIBUS";
pub(super) const EXTERNAL_OMNIBUS_ID: Uuid = uuid!("10000000-1000-0000-0000-000000000000");

pub(super) const STABLESATS_BTC_WALLET: &str = "STABLESATS_BTC_WALLET";
pub(super) const STABLESATS_BTC_WALLET_ID: Uuid = uuid!("20000000-2000-0000-0000-000000000000");

pub(super) const STABLESATS_OMNIBUS: &str = "STABLESATS_OMNIBUS";
pub(super) const STABLESATS_OMNIBUS_ID: Uuid = uuid!("20000000-1000-0000-0000-000000000000");

pub(super) const STABLESATS_LIABILITY: &str = "STABLESATS_LIABILITY";
pub(super) const STABLESATS_LIABILITY_ID: Uuid = uuid!("20000000-2100-0000-0000-000000000000");

// todo: change to a better uuid pattern
pub(super) const EXCHANGE_POSITION_OMNIBUS_CODE: &str = "EXCHANGE_POSITION_OMNIBUS";
pub(super) const EXCHANGE_POSITION_OMNIBUS_ID: Uuid = uuid!("10000000-1000-0000-0000-000000000001");

pub(super) const OKEX_POSITION_CODE: &str = "OKEX_POSITION";
pub(super) const OKEX_POSITION_ID: Uuid = uuid!("10000000-1000-0000-0000-000000000002");

pub const SATS_PER_BTC: Decimal = dec!(100_000_000);
pub const CENTS_PER_USD: Decimal = dec!(100);
