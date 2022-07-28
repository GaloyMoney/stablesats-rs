use fred::prelude::*;

use super::error::PublisherError;
use super::message::*;

pub struct Publisher {
    client: RedisClient,
}

impl Publisher {
    pub async fn new() -> Result<Self, PublisherError> {
        let config = RedisConfig::default();
        let client = RedisClient::new(config.clone());
        let _ = client.connect(None);
        let _ = client
            .wait_for_connect()
            .await
            .map_err(|e| PublisherError::InitialConnectionError(e))?;
        Ok(Self { client })
    }

    pub async fn publish<P: MessagePayload>(&self, payload: P) -> Result<(), PublisherError> {
        let payload_str = serde_json::to_string(&Envelope::new(payload))?;
        let _ = self
            .client
            .publish(<P as MessagePayload>::channel(), payload_str)
            .await?;
        Ok(())
    }
}
