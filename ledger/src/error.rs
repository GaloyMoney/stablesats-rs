use thiserror::Error;

#[allow(clippy::large_enum_variant)]
#[derive(Error, Debug)]
pub enum LedgerError {
    #[error("LedgerError - SqlxLedger: {0}")]
    SqlxLedger(#[from] sqlx_ledger::SqlxLedgerError),
    #[error("LedgerError - Transaction not found")]
    TransactionNotFound,
    #[error("LedgerError - MissingTxMetadata")]
    MissingTxMetadata,
    #[error("LedgerError - NotFound: {0}")]
    ExpectedEntryNotFoundInTx(&'static str),
    #[error("LedgerError - SerdeJson: {0}")]
    SerdeJson(#[from] serde_json::Error),
}
