use thiserror::Error;

#[derive(Error, Debug)]
pub enum OkexClientError {
    #[error("OkexClientError: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("OkexClientError: {0}")]
    Deserialization(#[from] serde_json::Error),
    #[error("OkexClientError: {0}")]
    Header(#[from] reqwest::header::InvalidHeaderValue),
    #[error("OkexClientError: {code:?} - {msg:?}")]
    UnexpectedResponse { msg: String, code: String },
}
