use thiserror::Error;

use crate::{currency::CurrencyError, order_book_snapshot_cache::SnapshotCacheError};
use shared::pubsub::SubscriberError;

#[allow(clippy::large_enum_variant)]
#[derive(Error, Debug)]
pub enum PriceAppError {
    #[error("PriceAppError - CurrencyError: {0}")]
    CurrencyError(#[from] CurrencyError),
    #[error("PriceAppError - SubscriberError: {0}")]
    SubscriberError(#[from] SubscriberError),
    #[error("PriceAppError - SnapshotCacheError: {0}")]
    SnapshotCacheError(#[from] SnapshotCacheError),
    #[error("PriceAppError - FloatingPointConversion: {0}")]
    FloatingPointConversion(#[from] rust_decimal::Error),
}
