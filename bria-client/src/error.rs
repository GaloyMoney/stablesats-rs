use thiserror::Error;
use tonic::{metadata::errors::InvalidMetadataValue, transport};

#[derive(Error, Debug)]
pub enum BriaClientError {
    #[error("Couldn't connect to bria at url: {0}")]
    ConnectionError(#[from] transport::Error),
    #[error("Couldn't create MetadataValue")]
    CouldNotInjectApiKey(#[from] InvalidMetadataValue),
    #[error("Could not parse Send Onchain Payment Metadata: {0}")]
    CouldNotParseSendOnchainPaymentMetadata(#[from] serde_json::Error),
    #[error("Could not convert Satoshis to u64")]
    CouldNotConvertSatoshisToU64,
    #[error("Tonic Error: {0}")]
    TonicError(#[from] tonic::Status),
}
