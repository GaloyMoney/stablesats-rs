use thiserror::Error;

use shared::pubsub::SubscriberError;

#[derive(Error, Debug)]
pub enum HedgingError {
    #[error("HedgingError: {0}")]
    PubSub(#[from] SubscriberError),
}
