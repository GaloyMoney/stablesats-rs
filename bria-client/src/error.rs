use thiserror::Error;

#[derive(Error, Debug)]
pub enum BriaClientError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("Bria key cannot be empty")]
    EmptyKeyError,
    #[error("Invalid metadata value: {0}")]
    InvalidMetadataValue(String),
    #[error("Couldn't find address for the given external_id")]
    AddressNotFoundError,
}
