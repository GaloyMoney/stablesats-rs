use thiserror::Error;

#[derive(Error, Debug)]
pub enum GaloyClientError {
    #[error("GaloyClientError - Reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("GaloyClientError - InvalidHeaderValue: {0}")]
    Header(#[from] reqwest::header::InvalidHeaderValue),
    #[error("GaloyClientError - GrqphqlApi: {0}")]
    GraphQLApi(String),
    #[error("GaloyClientError - Authentication: {0}")]
    Authentication(String),
}
