use fred::prelude::*;

use super::config::*;
use super::error::PublisherError;
use super::message::*;

pub struct Publisher {
    client: RedisClient,
}

impl Publisher {
    pub async fn new(PubSubConfig { host }: PubSubConfig) -> Result<Self, PublisherError> {
        let mut config = RedisConfig::default();
        if let Some(host) = host {
            config.server = ServerConfig::new_centralized(host, 6379);
        }
        let client = RedisClient::new(config);
        let _ = client.connect(None);
        client
            .wait_for_connect()
            .await
            .map_err(PublisherError::InitialConnection)?;
        Ok(Self { client })
    }

    pub async fn publish<P: MessagePayload>(&self, payload: P) -> Result<(), PublisherError> {
        let payload_str = serde_json::to_string(&Envelope::new(payload))?;
        self.client
            .publish(<P as MessagePayload>::channel(), payload_str)
            .await?;
        Ok(())
    }
}
