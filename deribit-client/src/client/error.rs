use thiserror::Error;

#[derive(Error, Debug)]
pub enum DeribitClientError {
    #[error("DeribitClientError - Reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("DeribitClientError - InvalidHeaderValue: {0}")]
    Header(#[from] reqwest::header::InvalidHeaderValue),
    #[error("DeribitClientError - SerdeJson: {0}")]
    Deserialization(#[from] serde_json::Error),
    #[error("DeribitClientError - DecimalConversion: {0}")]
    DecimalConversion(#[from] rust_decimal::Error),

    #[error("DeribitClientError - CannotConvertOrderStateFromStr")]
    CannotConvertOrderStateFromStr,

    #[error("DeribitClientError - UnexpectedResponse: {code:?} - {msg:?}")]
    UnexpectedResponse { msg: String, code: i64 },
    #[error("DeribitClientError - RequestParametersError: {code:?} - {msg:?}")]
    RequestParametersError { msg: String, code: i64 },
    #[error("DeribitClientError - NoLastPriceAvailable")]
    NoLastPriceAvailable,
}

impl From<(String, i64)> for DeribitClientError {
    fn from((msg, code): (String, i64)) -> Self {
        match code {
            10020 => DeribitClientError::RequestParametersError { msg, code },
            _ => DeribitClientError::UnexpectedResponse { msg, code },
        }
    }
}
