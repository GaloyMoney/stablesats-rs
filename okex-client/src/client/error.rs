use thiserror::Error;

#[allow(clippy::large_enum_variant)]
#[allow(clippy::enum_variant_names)]
#[derive(Error, Debug)]
pub enum OkexClientError {
    #[error("Error generating access signature")]
    AccessSignatureError,
    #[error("Error generating authentication headers")]
    AuthHeadersError,
    #[error("Error getting a response from Okex: {0}")]
    ResponseError(#[from] reqwest::Error),
    #[error("Encountered a (de)serializing error: {0}")]
    SerializationError(#[from] serde_json::error::Error),
    #[error("Encountered a (de)serializing error: {0}")]
    HeaderCreationError(#[from] reqwest::header::InvalidHeaderValue),
}
