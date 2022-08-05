use thiserror::Error;

#[derive(Error, Debug)]
pub enum OkexClientError {
    #[error("Error generating access signature")]
    AccessSignatureError,
    #[error("Error generating authentication headers")]
    AuthHeadersError,
}
