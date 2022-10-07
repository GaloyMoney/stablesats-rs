use std::collections::HashMap;

use graphql_client::Location;
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
    #[error("GaloyClientError - GraphQLNested {{ message: {message:?}, path: {path:?} }}")]
    GraphQLNested {
        message: String,
        path: Option<Vec<Option<String>>>,
    },
    #[error(
        "GaloyClientError - GraphQLTopLevel {{ message: {message:?}, path: {path:?}, locations: {locations:?}, extensions: {extensions:?} }}"
    )]
    GraphQLTopLevel {
        message: String,
        path: PathString,
        locations: Option<Vec<Location>>,
        extensions: Option<HashMap<String, Value, RandomState>>,
    },
    #[error("GaloyClientError - Authentication: {0}")]
    Authentication(String),
    #[error("GaloyClientError - Serde: {0}")]
    Serde(#[from] serde_json::Error),
}
