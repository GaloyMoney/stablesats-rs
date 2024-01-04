use thiserror::Error;

#[allow(clippy::large_enum_variant)]
#[derive(Error, Debug)]
pub enum LedgerError {
    #[error("LedgerError - SqlxLedger: {0}")]
    SqlxLedger(#[from] sqlx_ledger::SqlxLedgerError),
    #[error("HedgingError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
}
