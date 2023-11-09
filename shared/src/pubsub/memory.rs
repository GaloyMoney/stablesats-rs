use futures::{channel::mpsc::*, stream::StreamExt, SinkExt};
use governor::{clock::DefaultClock, state::keyed::DefaultKeyedStateStore, Quota, RateLimiter};
use tokio::sync::{broadcast, RwLock};
use tracing::instrument;

use std::{num::NonZeroU32, sync::Arc};

use super::message::*;
use crate::{health::HealthCheckResponse, time::TimeStamp};

const MAX_BURST: u32 = 1;

pub fn channel<P: MessagePayload>(
    rate_limit_interval: chrono::Duration,
) -> (Publisher<P>, Subscriber<P>) {
    let (tx, rx) = broadcast::channel(1);
    let rate_limiter = Arc::new(RateLimiter::keyed(
        Quota::with_period(
            rate_limit_interval
                .to_std()
                .expect("Could not convert to std"),
        )
        .expect("couldn't create quota")
        .allow_burst(NonZeroU32::new(MAX_BURST).unwrap()),
    ));
    let last_msg_timestamp = Arc::new(RwLock::new(None));
    let ts = Arc::clone(&last_msg_timestamp);
    let (timestamp_sender, mut rcv) = unbounded();
    tokio::spawn(async move {
        while let Some(timestamp) = rcv.next().await {
            let mut last_ts = ts.write().await;
            if last_ts.unwrap_or(timestamp) <= timestamp {
                *last_ts = Some(timestamp);
            }
        }
    });
    (
        Publisher {
            inner: tx,
            rate_limiter,
        },
        Subscriber {
            inner: rx,
            last_msg_timestamp,
            timestamp_sender,
        },
    )
}

#[derive(Clone)]
pub struct Publisher<P: MessagePayload> {
    inner: broadcast::Sender<Envelope<P>>,
    rate_limiter:
        Arc<RateLimiter<&'static str, DefaultKeyedStateStore<&'static str>, DefaultClock>>,
}

impl<P: MessagePayload> Publisher<P> {
    pub async fn throttle_publish(
        &self,
        throttle_key: &'static str,
        payload: P,
    ) -> Result<bool, broadcast::error::SendError<Envelope<P>>> {
        if self.rate_limiter.check_key(&throttle_key).is_ok() {
            self.publish(payload).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    #[instrument(name = "pubsub.memory.publish", level = "trace", skip_all, fields(correlation_id, payload_type, payload_json, error, error.message), err)]
    pub async fn publish(
        &self,
        payload: P,
    ) -> Result<(), broadcast::error::SendError<Envelope<P>>> {
        let span = tracing::Span::current();
        span.record(
            "payload_type",
            &tracing::field::display(<P as MessagePayload>::message_type()),
        );
        span.record(
            "payload_json",
            &tracing::field::display(serde_json::to_string(&payload).expect("Could not serialize")),
        );
        let msg = Envelope::new(payload);
        span.record(
            "published_at",
            &tracing::field::display(&msg.meta.published_at),
        );

        crate::tracing::record_error(tracing::Level::WARN, || async move { self.inner.send(msg) })
            .await?;
        Ok(())
    }
}

pub struct Subscriber<P: MessagePayload> {
    inner: broadcast::Receiver<Envelope<P>>,
    last_msg_timestamp: Arc<RwLock<Option<TimeStamp>>>,
    timestamp_sender: UnboundedSender<TimeStamp>,
}

impl<P: MessagePayload> Subscriber<P> {
    pub fn resubscribe(&self) -> Self {
        Self {
            inner: self.inner.resubscribe(),
            last_msg_timestamp: Arc::clone(&self.last_msg_timestamp),
            timestamp_sender: self.timestamp_sender.clone(),
        }
    }

    pub async fn next(&mut self) -> Option<Envelope<P>> {
        loop {
            match self.inner.recv().await {
                Ok(msg) => {
                    let _ = self.timestamp_sender.send(msg.meta.published_at).await;
                    return Some(msg);
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => return None,
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
            }
        }
    }

    pub async fn healthy(&self, largest_msg_delay: chrono::Duration) -> HealthCheckResponse {
        let last_msg_timestamp = self.last_msg_timestamp.read().await;
        if let Some(time_since) = last_msg_timestamp.map(|ts| ts.duration_since()) {
            if time_since <= largest_msg_delay {
                Ok(())
            } else {
                Err(format!(
                    "No '{}' messages received in the last {} seconds",
                    <P as MessagePayload>::message_type(),
                    time_since.num_seconds()
                ))
            }
        } else {
            Err(format!(
                "No '{}' messages received",
                <P as MessagePayload>::message_type()
            ))
        }
    }
}
