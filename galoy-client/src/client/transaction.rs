use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

use super::stablesats_transactions_list;

#[derive(Debug)]
pub struct TxCursor(pub(super) String);
impl From<String> for TxCursor {
    fn from(cursor: String) -> Self {
        Self(cursor)
    }
}

pub type GaloyTransactionEdge =
    stablesats_transactions_list::StablesatsTransactionsListMeDefaultAccountTransactionsEdges;
pub type GaloyTransactionNode =
    stablesats_transactions_list::StablesatsTransactionsListMeDefaultAccountTransactionsEdgesNode;

pub type SettlementCurrency = stablesats_transactions_list::WalletCurrency;
pub type TxStatus = stablesats_transactions_list::TxStatus;

#[derive(Debug)]
pub struct GaloyTransaction {
    pub cursor: TxCursor,
    pub created_at: DateTime<Utc>,
    pub id: String,
    pub settlement_amount: Decimal,
    pub settlement_currency: SettlementCurrency,
    pub cents_per_unit: Decimal,
    pub amount_in_usd_cents: Decimal,
    pub status: TxStatus,
}

#[derive(Debug)]
pub struct GaloyTransactions {
    pub cursor: Option<TxCursor>,
    pub list: Vec<GaloyTransaction>,
    pub has_more: bool,
}
