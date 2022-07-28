use fred::prelude::RedisError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PublisherError {
    #[error("Publisher couldn't connect: {0}")]
    InitialConnectionError(RedisError),
    #[error("Publisher couldn't serialize: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Publisher encounted unknown error: {0}")]
    UnknownRedisError(#[from] RedisError),
}

#[derive(Error, Debug)]
pub enum SubscriberError {
    #[error("{0}")]
    InitialConnectionError(RedisError),
    #[error("{0}")]
    DeserializationError(#[from] serde_json::Error),
    #[error("{0}")]
    UnknownRedisError(#[from] RedisError),
}
