use fred::{clients::SubscriberClient, prelude::*};
use futures::stream::{Stream, StreamExt};
use std::sync::mpsc::{channel, Receiver, Sender};

use super::config::*;
use super::error::SubscriberError;
use super::message::*;

pub struct Subscriber {
    client: SubscriberClient,
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
        Ok(Self { client })
    }

    pub async fn subscribe<M: MessagePayload>(
        &self,
    ) -> Result<std::pin::Pin<Box<dyn Stream<Item = Envelope<M>> + Send>>, SubscriberError> {
        let message_stream = self.client.on_message();
        self.client
            .subscribe(<M as MessagePayload>::channel())
            .await?;
        Ok(Box::pin(message_stream.filter_map(
            |(channel, value)| async move {
                if channel == <M as MessagePayload>::channel() {
                    if let RedisValue::String(v) = value {
                        if let Ok(msg) = serde_json::from_str::<Envelope<M>>(&v) {
                            if msg.payload_type == <M as MessagePayload>::message_type() {
                                return Some(msg);
                            }
                        }
                    }
                }
                None
            },
        )))
    }

    pub async fn subscribe_alt<M: MessagePayload>(
        &self,
    ) -> Result<Receiver<std::pin::Pin<Box<dyn Stream<Item = Envelope<M>> + Send>>>, SubscriberError>
    {
        let (tx, rx) = channel();

        // when an initial connection or reconnection happens, read data from pubsub channel
        // and send to mpsc channel
        let sender = tx.clone();
        let _jh = tokio::spawn(self.client.on_reconnect().for_each(
            move |client: SubscriberClient| async move {
                // Listen for event when there is a message
                let message_stream = client.on_message();
                // Subscribe to channel
                client.subscribe(<M as MessagePayload>::channel()).await?;

                // Retrieve message payload from subscribed channel
                let msg = Box::pin(message_stream.filter_map(|(channel, value)| async move {
                    if channel == <M as MessagePayload>::channel() {
                        if let RedisValue::String(v) = value {
                            if let Ok(msg) = serde_json::from_str::<Envelope<M>>(&v) {
                                if msg.payload_type == <M as MessagePayload>::message_type() {
                                    return Some(msg);
                                }
                            }
                        }
                    }
                    None
                }));

                // Send message to mpsc channel
                sender.send(msg)
            },
        ));

        // Connect to server if connection is down
        while !self.client.is_connected() {
            let _connect_jh = self.client.connect(None);
            let _wait_for_conn = self.client.wait_for_connect().await?;
        }

        Ok(rx)
    }
}
