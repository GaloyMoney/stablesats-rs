use thiserror::Error;

use super::okex_response::ResponseWithoutData;

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

impl From<ResponseWithoutData> for OkexClientError {
    fn from(response: ResponseWithoutData) -> Self {
        OkexClientError::UnexpectedResponse {
            msg: response.msg,
            code: response.code,
        }
    }
}
