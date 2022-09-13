use thiserror::Error;

#[derive(Error, Debug)]
pub enum OkexClientError {
    #[error("OkexClientError - Reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("OkexClientError - SerdeJson: {0}")]
    Deserialization(#[from] serde_json::Error),
    #[error("OkexClientError - InvalidHeaderValue: {0}")]
    Header(#[from] reqwest::header::InvalidHeaderValue),
    #[error("OkexClientError - UnexpectedResponse: {code:?} - {msg:?}")]
    UnexpectedResponse { msg: String, code: String },
    #[error("OkexClientError - DecimalConversion: {0}")]
    DecimalConversion(#[from] rust_decimal::Error),
    #[error("OkexClientError - MosconfiguredAccount: {0}")]
    MisconfiguredAccount(String),
}
