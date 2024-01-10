use thiserror::Error;

use shared::sqlxmq::JobExecutionError;

#[allow(clippy::large_enum_variant)]
#[derive(Error, Debug)]
pub enum HedgingError {
    #[error("HedgingError - SerdeJson: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("HedgingError - Sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("HedgingError - Migrate: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error("HedgingError - OkexClient: {0}")]
    OkexClient(#[from] okex_client::OkexClientError),
    #[error("HedgingError - GaloyClient: {0}")]
    GaloyClient(#[from] galoy_client::GaloyClientError),
    #[error("HedgingError - NoJobDataPresent")]
    NoJobDataPresent,
    #[error("UserTradesError - Leger: {0}")]
    Ledger(#[from] ledger::LedgerError),
    #[error("BriaClientError - BriaClient: {0}")]
    BriaClient(#[from] bria_client::BriaClientError),
}

impl JobExecutionError for HedgingError {}
