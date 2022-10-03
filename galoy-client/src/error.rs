use std::collections::HashMap;

use graphql_client::{Location, PathFragment};
use serde::Deserialize;
use serde_json::Value;
use std::collections::hash_map::RandomState;
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

#[derive(Debug, Deserialize)]
pub struct InnerError {
    pub message: String,
    pub path: Option<Vec<Option<String>>>,
}

#[derive(Debug, Deserialize)]
pub struct TopLevelError {
    pub message: String,
    pub path: std::option::Option<Vec<PathFragment>>,
    pub location: Option<Vec<Location>>,
    pub extensions: Option<HashMap<String, Value, RandomState>>,
}

impl TopLevelError {
    pub fn create(errors: Vec<graphql_client::Error>) -> Result<Self, GaloyClientError> {
        let mut errors_list = Vec::new();

        for error in errors {
            let err = Self::from(error);
            errors_list.push(err)
        }

        Err(GaloyClientError::GraphQLApi(format!("{:#?}", errors_list)))
    }
}
