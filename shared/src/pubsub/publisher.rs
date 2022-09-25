use fred::prelude::*;
use std::time::Duration;
use tracing::instrument;

use super::config::*;
use super::error::PublisherError;
use super::message::*;

use governor::{
    clock::DefaultClock, state::keyed::DefaultKeyedStateStore, Jitter, Quota, RateLimiter,
};
use std::num::NonZeroU32;

lazy_static::lazy_static! {
    static ref LIMITER: RateLimiter<&'static str, DefaultKeyedStateStore<&'static str>, DefaultClock>  = RateLimiter::keyed(Quota::per_minute(NonZeroU32::new(30).unwrap()));
}

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

    pub async fn rate_limit_publisher(&self, key: &'static str) -> &RedisClient {
        let jitter = Jitter::new(Duration::from_secs(2), Duration::from_secs(1));
        LIMITER.until_key_ready_with_jitter(&key, jitter).await;
        &self.client
    }

    /// Throttles the publishing of price ticks
    pub async fn publish_price<P: MessagePayload>(&self, payload: P) -> Result<(), PublisherError> {
        let msg = Envelope::new(payload);
        let payload_str = serde_json::to_string(&msg)?;

        let _throttled_publishing = self
            .rate_limit_publisher(<P as MessagePayload>::channel())
            .await
            .publish(<P as MessagePayload>::channel(), payload_str)
            .await?;

        Ok(())
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
        crate::tracing::record_error(tracing::Level::WARN, || async move {
            self.client
                .publish(<P as MessagePayload>::channel(), payload_str)
                .await
        })
        .await?;
        Ok(())
    }
}
