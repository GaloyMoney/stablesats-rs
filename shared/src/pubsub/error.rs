use thiserror::Error;

#[derive(Error, Debug)]
pub enum PublisherError {
    #[error("Publisher couldn't serialize: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Error, Debug)]
pub enum SubscriberError {
    #[error("Subscriber couldn't deserialize: {0}")]
    Deserialization(#[from] serde_json::Error),
}
