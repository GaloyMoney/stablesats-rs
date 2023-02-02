use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::{uuid, Uuid};

// Templates
pub(super) const USER_BUYS_USD_CODE: &str = "USER_BUYS_USD";
pub(super) const USER_SELLS_USD_CODE: &str = "USER_SELLS_USD";

// Journal
pub(super) const STABLESATS_JOURNAL_NAME: &str = "Stablesats";
pub(super) const STABLESATS_JOURNAL_ID: Uuid = uuid!("00000000-0000-0000-0000-000000000001");

// Accounts
pub(super) const EXTERNAL_OMNIBUS_CODE: &str = "EXTERNAL_OMNIBUS";
pub(super) const EXTERNAL_OMNIBUS_ID: Uuid = uuid!("00000000-0000-0000-0000-000000000001");

pub(super) const STABLESATS_BTC_WALLET: &str = "STABLESATS_BTC_WALLET";
pub(super) const STABLESATS_BTC_WALLET_ID: Uuid = uuid!("10000000-0000-0000-0000-000000000001");

pub(super) const STABLESATS_OMNIBUS: &str = "STABLESATS_OMNIBUS";
pub(super) const STABLESATS_OMNIBUS_ID: Uuid = uuid!("20000000-0000-0000-0000-000000000001");

pub(super) const STABLESATS_LIABILITY: &str = "STABLESATS_LIABILITY";
pub(super) const STABLESATS_LIABILITY_ID: Uuid = uuid!("20000000-0000-0000-0000-000000000002");

pub const SATS_PER_BTC: Decimal = dec!(100_000_000);
pub const CENTS_PER_USD: Decimal = dec!(100);
