use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

use super::stablesats_transactions_list;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxCursor(pub(super) String);
impl From<String> for TxCursor {
    fn from(cursor: String) -> Self {
        Self(cursor)
    }
}
impl From<TxCursor> for String {
    fn from(cursor: TxCursor) -> Self {
        cursor.0
    }
}

pub type GaloyTransactionEdge =
    stablesats_transactions_list::StablesatsTransactionsListMeDefaultAccountTransactionsEdges;
pub type GaloyTransactionNode =
    stablesats_transactions_list::StablesatsTransactionsListMeDefaultAccountTransactionsEdgesNode;
pub type TxDirection = stablesats_transactions_list::TxDirection;

pub type SettlementCurrency = stablesats_transactions_list::WalletCurrency;
impl std::fmt::Display for SettlementCurrency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BTC => write!(f, "BTC"),
            Self::USD => write!(f, "USD"),
            Self::Other(other) => write!(f, "{other}"),
        }
    }
}
impl std::str::FromStr for SettlementCurrency {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BTC" => Ok(Self::BTC),
            "USD" => Ok(Self::USD),
            other => Ok(Self::Other(other.to_string())),
        }
    }
}

pub type TxStatus = stablesats_transactions_list::TxStatus;
pub type SettlementMethod = stablesats_transactions_list::StablesatsTransactionsListMeDefaultAccountTransactionsEdgesNodeSettlementVia;
impl std::fmt::Display for SettlementMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SettlementViaIntraLedger => write!(f, "SettlementViaIntraLedger"),
            Self::SettlementViaOnChain => write!(f, "SettlementViaOnChain"),
            Self::SettlementViaLn => write!(f, "SettlementViaLn"),
        }
    }
}

impl std::fmt::Display for TxDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RECEIVE => write!(f, "RECEIVE"),
            Self::SEND => write!(f, "SEND"),
            Self::Other(other) => write!(f, "{other}"),
        }
    }
}

#[derive(Debug)]
pub struct GaloyTransaction {
    pub id: String,
    pub cursor: TxCursor,
    pub settlement_amount: Decimal,
    pub settlement_currency: SettlementCurrency,
    pub settlement_method: SettlementMethod,
    pub memo: Option<String>,
    pub direction: TxDirection,
    pub cents_per_unit: Decimal,
    pub amount_in_usd_cents: Decimal,
    pub status: TxStatus,
    pub created_at: DateTime<Utc>,
}

impl std::fmt::Display for GaloyTransaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "id: {}, ", self.id)?;
        write!(f, "cursor: {}, ", self.cursor.0)?;
        write!(f, "settlement_amount: {}, ", self.settlement_amount)?;
        write!(f, "settlement_currency: {}, ", self.settlement_currency)?;
        write!(f, "settlement_method: {}, ", self.settlement_method)?;
        write!(f, "memo: {}, ", self.memo.as_ref().unwrap())?;
        write!(f, "direction: {}, ", self.direction)?;
        write!(f, "cents_per_unit: {}, ", self.cents_per_unit)?;
        write!(f, "amount_in_usd_cents: {}, ", self.amount_in_usd_cents)?;
        write!(f, "created_at: {}, ", self.created_at)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct GaloyTransactions {
    pub cursor: Option<TxCursor>,
    pub list: Vec<GaloyTransaction>,
    pub has_more: bool,
}

impl std::fmt::Display for GaloyTransactions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "cursor: {}, ", self.cursor.as_ref().unwrap().0)?;
        writeln!(f, "[")?;
        for tx in &self.list {
            writeln!(f, "{}", tx)?;
        }
        writeln!(f, "]")?;
        writeln!(f, "has_more: {}, ", self.has_more)?;
        Ok(())
    }
}
