use fred::prelude::*;
use tracing::instrument;

use super::config::*;
use super::error::PublisherError;
use super::message::*;

pub struct Publisher {
    client: RedisClient,
}

impl Publisher {
    pub async fn new(config: PubSubConfig) -> Result<Self, PublisherError> {
        let client = RedisClient::new(config.into());
        let _ = client.connect(None);
        client
            .wait_for_connect()
            .await
            .map_err(PublisherError::InitialConnection)?;
        Ok(Self { client })
    }

    #[instrument(skip(self), fields(correlation_id), err)]
    pub async fn publish<P: MessagePayload>(&self, payload: P) -> Result<(), PublisherError> {
        let payload_str = serde_json::to_string(&Envelope::new(payload))?;
        self.client
            .publish(<P as MessagePayload>::channel(), payload_str)
            .await?;
        Ok(())
    }
}
