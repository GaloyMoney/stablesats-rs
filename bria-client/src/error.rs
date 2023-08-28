use thiserror::Error;

#[derive(Error, Debug)]
pub enum BriaClientError {
    #[error("Couldn't connect to bria at url: {0}")]
    ConnectionError(String),
    #[error("Bria key cannot be empty")]
    EmptyKeyError,
    #[error("Couldn't create MetadataValue")]
    CouldNotCreateMetadataValue,
    #[error("Couldn't find address for the given external_id")]
    AddressNotFoundError,
    #[error("Couldn't generate a new address: {0}")]
    CouldNotGenerateNewAddress(String),
    #[error("Couldn't send onchain payment: {0}")]
    CouldNotSendOnchainPayment(String),
    #[error("Could not parse Send Onchain Payment Metadata: {0}")]
    CouldNotParseSendOnchainPaymentMetadata(serde_json::Error),
}

impl From<serde_json::Error> for BriaClientError {
    fn from(err: serde_json::Error) -> BriaClientError {
        BriaClientError::CouldNotParseSendOnchainPaymentMetadata(err)
    }
}
