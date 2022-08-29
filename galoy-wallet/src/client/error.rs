use thiserror::Error;

#[derive(Error, Debug)]
pub enum GaloyClientError {
    #[error("GaloyWalletError: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("GaloyWalletError: {0}")]
    UnknownResponse(String),
    #[error("GaloyWalletError: {0}")]
    Header(#[from] reqwest::header::InvalidHeaderValue),
    #[error("GaloyWalletError: {0}")]
    GrapqQlApi(String),
    #[error("GaloyWalletError: {0}")]
    AuthenticationToken(String),
}
