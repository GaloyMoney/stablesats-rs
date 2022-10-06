use std::collections::HashMap;

use graphql_client::Location;
// use serde::Deserialize;
use serde_json::Value;
use std::collections::hash_map::RandomState;
use thiserror::Error;

use crate::PathString;

#[derive(Error, Debug)]
pub enum GaloyClientError {
    #[error("GaloyClientError - Reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("GaloyClientError - InvalidHeaderValue: {0}")]
    Header(#[from] reqwest::header::InvalidHeaderValue),
    #[error("GaloyClientError - GrqphqlApi{{ msg: {message:?}, path: {path:?}, loc: {location:?}, ext: {extensions:?} }}")]
    GraphQLApi {
        message: String,
        path: PathString,
        location: Option<Vec<Location>>,
        extensions: Option<HashMap<String, Value, RandomState>>,
    },
    #[error("GaloyClientError - Authentication: {0}")]
    Authentication(String),
}
