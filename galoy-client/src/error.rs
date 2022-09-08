use thiserror::Error;

#[derive(Error, Debug)]
pub enum GaloyClientError {
    #[error("GaloyClientError: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("GaloyClientError: {0}")]
    Header(#[from] reqwest::header::InvalidHeaderValue),
    #[error("GaloyClientError: {0}")]
    GrapqQlApi(String),
    #[error("GaloyClientError: {0}")]
    Authentication(String),
    #[error("GaloyClientError: {0}")]
    TransactionUnification(String),
}
