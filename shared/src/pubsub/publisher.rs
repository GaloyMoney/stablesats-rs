use fred::prelude::*;
use tracing::instrument;

use super::config::*;
use super::error::PublisherError;
use super::message::*;

#[derive(Clone)]
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

    #[instrument(skip_all, fields(correlation_id, payload_type, payload_json, error, error.message), err)]
    pub async fn publish<P: MessagePayload>(&self, payload: P) -> Result<(), PublisherError> {
        let span = tracing::Span::current();
        span.record(
            "payload_type",
            &tracing::field::display(<P as MessagePayload>::message_type()),
        );
        span.record(
            "payload_json",
            &tracing::field::display(serde_json::to_string(&payload)?),
        );
        let msg = Envelope::new(payload);
        span.record(
            "published_at",
            &tracing::field::display(&msg.meta.published_at),
        );

        let payload_str = serde_json::to_string(&msg)?;
        crate::tracing::record_error(|| async move {
            self.client
                .publish(<P as MessagePayload>::channel(), payload_str)
                .await
        })
        .await?;
        Ok(())
    }
}
