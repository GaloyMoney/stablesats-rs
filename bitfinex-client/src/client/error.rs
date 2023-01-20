use thiserror::Error;

#[derive(Error, Debug)]
pub enum BitfinexClientError {
    #[error("BitfinexClientError - Reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("BitfinexClientError - InvalidHeaderValue: {0}")]
    Header(#[from] reqwest::header::InvalidHeaderValue),
    #[error("BitfinexClientError - SerdeJson: {0}")]
    Deserialization(#[from] serde_json::Error),
    #[error("BitfinexClientError - DecimalConversion: {0}")]
    DecimalConversion(#[from] rust_decimal::Error),

    #[error("BitfinexClientError - UnexpectedResponse: {code:?} - {msg:?}")]
    UnexpectedResponse { msg: String, code: u32 },
    #[error("BitfinexClientError - RequestParametersError: {code:?} - {msg:?}")]
    RequestParametersError { msg: String, code: u32 },
    #[error("BitfinexClientError - NoLastPriceAvailable")]
    NoLastPriceAvailable,
}

impl From<(String, u32)> for BitfinexClientError {
    fn from((msg, code): (String, u32)) -> Self {
        match code {
            10020 => BitfinexClientError::RequestParametersError { msg, code },
            _ => BitfinexClientError::UnexpectedResponse { msg, code },
        }
    }
}
