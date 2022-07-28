use fred::{clients::SubscriberClient, prelude::*};
use futures::stream::{Stream, StreamExt};

use super::error::SubscriberError;
use super::message::*;

pub struct Subscriber {
    client: SubscriberClient,
}

impl Subscriber {
    pub async fn new() -> Result<Self, SubscriberError> {
        let config = RedisConfig::default();
        let client = SubscriberClient::new(config.clone());
        let _ = client.connect(None);
        let _ = client
            .wait_for_connect()
            .await
            .map_err(|e| SubscriberError::InitialConnectionError(e))?;
        let _ = client.manage_subscriptions();
        Ok(Self { client })
    }

    pub async fn subscribe<M: MessagePayload>(
        &self,
    ) -> Result<impl Stream<Item = Result<Envelope<M>, SubscriberError>>, SubscriberError> {
        let message_stream = self.client.on_message();
        self.client
            .subscribe(<M as MessagePayload>::channel())
            .await?;
        Ok(message_stream.filter_map(|(channel, value)| async move {
            if channel == <M as MessagePayload>::channel() {
                if let RedisValue::String(v) = value {
                    return Some(
                        serde_json::from_str::<Envelope<M>>(&v).map_err(SubscriberError::from),
                    );
                }
            }
            None
        }))
    }
}
