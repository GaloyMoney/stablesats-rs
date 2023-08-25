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
}
