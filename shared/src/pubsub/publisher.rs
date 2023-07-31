use fred::prelude::*;
use tracing::instrument;

use super::config::*;
use super::error::PublisherError;
use super::message::*;

use governor::{clock::DefaultClock, state::keyed::DefaultKeyedStateStore, Quota, RateLimiter};
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
        let rate_limiter = Arc::new(RateLimiter::keyed(
            Quota::with_period(config.rate_limit_interval)
                .expect("couldn't create quota")
                .allow_burst(NonZeroU32::new(MAX_BURST).unwrap()),
        ));
        let client = RedisClient::new(config.into(), None, None);
        tokio::spawn(client.connect());
        client
            .wait_for_connect()
            .await
            .map_err(PublisherError::InitialConnection)?;

        Ok(Self {
            client,
            rate_limiter,
        })
    }

    /// Throttles the publishing of messages
    pub async fn throttle_publish<P: MessagePayload>(
        &self,
        payload: P,
    ) -> Result<bool, PublisherError> {
        let throttle_key = <P as MessagePayload>::message_type();
        if self.rate_limiter.check_key(&throttle_key).is_ok() {
            self.publish(payload).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    #[instrument(name = "pubsub.redis.publish", skip_all, fields(correlation_id, payload_type, payload_json, error, error.message), err)]
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
