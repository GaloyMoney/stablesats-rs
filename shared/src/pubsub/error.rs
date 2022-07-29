use fred::prelude::RedisError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PublisherError {
    #[error("Publisher couldn't connect: {0}")]
    InitialConnection(RedisError),
    #[error("Publisher couldn't serialize: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Publisher encountered unknown error: {0}")]
    UnknownRedisError(#[from] RedisError),
}

#[derive(Error, Debug)]
pub enum SubscriberError {
    #[error("Subscriber couldn't connect: {0}")]
    InitialConnection(RedisError),
    #[error("Subscriber couldn't deserialize: {0}")]
    Deserialization(#[from] serde_json::Error),
    #[error("Subscriber encountered unknown error: {0}")]
    UnknownRedisError(#[from] RedisError),
}
