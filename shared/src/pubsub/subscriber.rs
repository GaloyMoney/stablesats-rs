use fred::{clients::SubscriberClient, prelude::*};
use futures::stream::{Stream, StreamExt};

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
                        if let Ok(msg) = serde_json::from_str(&v) {
                            return Some(msg);
                        }
                    }
                }
                None
            },
        )))
    }
}
