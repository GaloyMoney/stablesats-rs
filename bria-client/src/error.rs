use thiserror::Error;

#[derive(Error, Debug)]
pub enum BriaClientError {
    #[error("Couldn't connect to bria at url: {0}")]
    ConnectionError(String),
    #[error("Couldn't create MetadataValue")]
    CouldNotCreateMetadataValue,
    #[error("Could not parse Send Onchain Payment Metadata: {0}")]
    CouldNotParseSendOnchainPaymentMetadata(serde_json::Error),
    #[error("Could not convert Satoshis to u64")]
    CouldNotConvertSatoshisToU64,
    #[error("Tonic Error: {0}")]
    TonicError(tonic::Status),
}

impl From<serde_json::Error> for BriaClientError {
    fn from(err: serde_json::Error) -> BriaClientError {
        BriaClientError::CouldNotParseSendOnchainPaymentMetadata(err)
    }
}

impl From<tonic::Status> for BriaClientError {
    fn from(err: tonic::Status) -> BriaClientError {
        BriaClientError::TonicError(err)
    }
}
