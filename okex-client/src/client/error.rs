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
    #[error("OkexClientError - ServiceUnavailable: {code:?} - {msg:?}")]
    ServiceUnavailable { msg: String, code: String },
    #[error("OkexClientError - OrderDoesNotExist")]
    OrderDoesNotExist,
    #[error("OkexClientError - DecimalConversion: {0}")]
    DecimalConversion(#[from] rust_decimal::Error),
    #[error("OkexClientError - MisconfiguredAccount: {0}")]
    MisconfiguredAccount(String),
}

impl From<(String, String)> for OkexClientError {
    fn from((msg, code): (String, String)) -> Self {
        match code.as_str() {
            "50001" => OkexClientError::ServiceUnavailable { msg, code },
            "51603" => OkexClientError::OrderDoesNotExist,
            _ => OkexClientError::UnexpectedResponse { msg, code },
        }
    }
}
