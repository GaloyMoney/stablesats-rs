use thiserror::Error;

#[derive(Error, Debug)]
pub enum BitfinexClientError {
    #[error("BitfinexClientError - Reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("BitfinexClientError - SerdeJson: {0}")]
    Deserialization(#[from] serde_json::Error),
    #[error("BitfinexClientError - InvalidHeaderValue: {0}")]
    Header(#[from] reqwest::header::InvalidHeaderValue),
    #[error("BitfinexClientError - UnexpectedResponse: {code:?} - {msg:?}")]
    UnexpectedResponse { msg: String, code: String },
    #[error("BitfinexClientError - ServiceUnavailable: {code:?} - {msg:?}")]
    ServiceUnavailable { msg: String, code: String },
    #[error("BitfinexClientError - RequestParametersError: {code:?} - {msg:?}")]
    RequestParametersError { msg: String, code: String },
    #[error("BitfinexClientError - OrderDoesNotExist")]
    OrderDoesNotExist,
    #[error("BitfinexClientError - ParameterClientIdError")]
    ParameterClientIdError,
    #[error("BitfinexClientError - WithdrawalIdDoesNotExist")]
    WithdrawalIdDoesNotExist,
    #[error("BitfinexClientError - NoLastPriceAvailable")]
    NoLastPriceAvailable,
    #[error("BitfinexClientError - DecimalConversion: {0}")]
    DecimalConversion(#[from] rust_decimal::Error),
    #[error("BitfinexClientError - MisconfiguredAccount: {0}")]
    MisconfiguredAccount(String),
}

impl From<(String, String)> for BitfinexClientError {
    fn from((msg, code): (String, String)) -> Self {
        match code.as_str() {
            "10020" => BitfinexClientError::RequestParametersError { msg, code },
            _ => BitfinexClientError::UnexpectedResponse { msg, code },
        }
    }
}
