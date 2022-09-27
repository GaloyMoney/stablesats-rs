use fred::prelude::*;
use std::time::Duration;
use tracing::instrument;

use super::config::*;
use super::error::PublisherError;
use super::message::*;

use governor::{
    clock::{DefaultClock, QuantaInstant},
    state::keyed::DefaultKeyedStateStore,
    NotUntil, Quota, RateLimiter,
};
use std::{num::NonZeroU32, sync::Arc};

const MAX_BURST: u32 = 1;

#[derive(Clone)]
pub struct Publisher {
    client: RedisClient,
    rate_limiter:
        Arc<RateLimiter<&'static str, DefaultKeyedStateStore<&'static str>, DefaultClock>>,
}

impl Publisher {
    pub async fn new(config: PubSubConfig) -> Result<Self, PublisherError> {
        let client = RedisClient::new(config.clone().into());
        let _ = client.connect(None);
        client
            .wait_for_connect()
            .await
            .map_err(PublisherError::InitialConnection)?;

        Ok(Self {
            client,
            rate_limiter: Arc::new(RateLimiter::keyed(
                Quota::with_period(Duration::from_secs(config.rate_limit_interval.unwrap()))
                    .unwrap()
                    .allow_burst(NonZeroU32::new(MAX_BURST).unwrap()),
            )),
        })
    }

    fn rate_limit_publisher(
        &self,
        key: &'static str,
    ) -> Result<&Publisher, NotUntil<QuantaInstant>> {
        self.rate_limiter.check_key(&key)?;
        Ok(self)
    }

    /// Throttles the publishing of messages
    pub async fn throttle_publish<P: MessagePayload>(
        &self,
        payload: P,
    ) -> Result<bool, PublisherError> {
        if let Ok(publisher) = self.rate_limit_publisher(<P as MessagePayload>::message_type()) {
            publisher.publish(payload).await?;
            Ok(true)
        } else {
            Ok(false)
        }
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
