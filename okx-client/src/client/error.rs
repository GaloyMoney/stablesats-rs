use thiserror::Error;

#[derive(Error, Debug)]
pub enum OkxClientError {
    #[error("OkxClientError - Reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("OkxClientError - SerdeJson: {0}")]
    Deserialization(#[from] serde_json::Error),
    #[error("OkxClientError - InvalidHeaderValue: {0}")]
    Header(#[from] reqwest::header::InvalidHeaderValue),
    #[error("OkxClientError - UnexpectedResponse: {code:?} - {msg:?}")]
    UnexpectedResponse { msg: String, code: String },
    #[error("OkxClientError - ServiceUnavailable: {code:?} - {msg:?}")]
    ServiceUnavailable { msg: String, code: String },
    #[error("OkxClientError - OrderDoesNotExist")]
    OrderDoesNotExist,
    #[error("OkxClientError - ParameterClientIdError")]
    ParameterClientIdError,
    #[error("OkxClientError - WithdrawalIdDoesNotExist")]
    WithdrawalIdDoesNotExist,
    #[error("OkxClientError - NoLastPriceAvailable")]
    NoLastPriceAvailable,
    #[error("OkxClientError - DecimalConversion: {0}")]
    DecimalConversion(#[from] rust_decimal::Error),
    #[error("OkxClientError - MisconfiguredAccount: {0}")]
    MisconfiguredAccount(String),
}

impl From<(String, String)> for OkxClientError {
    fn from((msg, code): (String, String)) -> Self {
        match code.as_str() {
            "50001" => OkxClientError::ServiceUnavailable { msg, code },
            "51000" => OkxClientError::ParameterClientIdError,
            "51603" => OkxClientError::OrderDoesNotExist,
            "58215" => OkxClientError::WithdrawalIdDoesNotExist,
            _ => OkxClientError::UnexpectedResponse { msg, code },
        }
    }
}
