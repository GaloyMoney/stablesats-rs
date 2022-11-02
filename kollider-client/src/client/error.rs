use data_encoding::DecodeError;
use reqwest::header::InvalidHeaderValue;
use serde_json::error::Error as SerdeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KolliderClientError {
    #[error("KolliderClientError - Reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("KolliderClientError - InvalidHeaderValue: {0}")]
    InvalidHeaderValue(#[from] InvalidHeaderValue),

    #[error("KolliderClientError - DecodeError: {0}")]
    DecodeError(#[from] DecodeError),

    #[error("KolliderClientError - SerdeError: {0}")]
    SerdeError(#[from] SerdeError),
}
