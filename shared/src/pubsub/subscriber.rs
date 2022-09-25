use fred::{clients::SubscriberClient, prelude::*};
use futures::{
    channel::{mpsc::*, oneshot},
    stream::StreamExt,
    SinkExt,
};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{config::*, error::SubscriberError, message::*};
use crate::{health::HealthCheckResponse, time::TimeStamp};

pub struct Subscriber {
    client: SubscriberClient,
    last_msg_timestamp: Arc<RwLock<Option<TimeStamp>>>,
    timestamp_sender: UnboundedSender<TimeStamp>,
}

impl Subscriber {
    pub async fn new(config: PubSubConfig) -> Result<Self, SubscriberError> {
        let client = SubscriberClient::new(config.into());
        let _ = client.connect(None);
        client
            .wait_for_connect()
            .await
            .map_err(SubscriberError::InitialConnection)?;
        let _ = client.manage_subscriptions();
        let last_msg_timestamp = Arc::new(RwLock::new(None));
        let ts = Arc::clone(&last_msg_timestamp);
        let (timestamp_sender, mut rcv) = unbounded();
        tokio::spawn(async move {
            while let Some(timestamp) = rcv.next().await {
                *ts.write().await = Some(timestamp);
            }
        });
        Ok(Self {
            client,
            last_msg_timestamp,
            timestamp_sender,
        })
    }

    pub async fn time_since_last_msg(&self) -> Option<chrono::Duration> {
        let last_msg_timestamp = self.last_msg_timestamp.read().await;
        last_msg_timestamp.map(|ts| ts.duration_since())
    }

    pub async fn report_health(
        &self,
        largest_msg_delay: chrono::Duration,
        report: oneshot::Sender<HealthCheckResponse>,
    ) {
        if let Some(time_since) = self.time_since_last_msg().await {
            if time_since <= largest_msg_delay {
                report
                    .send(Ok(()))
                    .expect("Couldn't respond to health check");
            } else {
                report
                    .send(Err(format!(
                        "No messages received in the last {} seconds",
                        time_since.num_seconds()
                    )))
                    .expect("Couldn't respond to health check");
            }
        } else {
            let _ = report.send(Err("No messages received".to_string()));
        }
    }

    pub async fn subscribe<M: MessagePayload>(
        &self,
    ) -> Result<Receiver<Envelope<M>>, SubscriberError> {
        let message_stream = self.client.on_message();
        self.client
            .subscribe(<M as MessagePayload>::channel())
            .await?;
        let (snd, recv) = channel(100);
        tokio::spawn(
            message_stream
                .filter_map(|(channel, value)| async move {
                    if channel == <M as MessagePayload>::channel() {
                        if let RedisValue::String(v) = value {
                            if let Ok(msg) = serde_json::from_str::<Envelope<M>>(&v) {
                                if msg.payload_type == <M as MessagePayload>::message_type() {
                                    return Some(Ok(msg));
                                }
                            }
                        }
                    }
                    None
                })
                .forward(
                    self.timestamp_sender
                        .clone()
                        .with(|msg: Envelope<M>| async move { Ok(msg.meta.published_at) })
                        .fanout(snd),
                ),
        );
        Ok(recv)
    }
}
